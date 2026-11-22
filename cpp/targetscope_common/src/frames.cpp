// doc/frame_transport.md
#include "targetscope/frames.hpp"

#include <spdlog/spdlog.h>

#include <iox2/iceoryx2.hpp>
#include <mutex>
#include <stdexcept>

namespace targetscope::frames {

namespace {

using Node = iox2::Node<iox2::ServiceType::Ipc>;
using Payload = iox2::bb::Slice<std::uint8_t>;

[[noreturn]] void fail(const std::string& what) { throw std::runtime_error("iceoryx2: " + what); }

template <class E>
std::string text(const E& e) {
    return std::to_string(static_cast<int>(e));
}

Node make_node() {
    auto node = iox2::NodeBuilder().config(iox2::Config()).create<iox2::ServiceType::Ipc>();
    if (!node.has_value()) fail("node: " + text(node.error()));
    return std::move(node.value());
}

// doc/frame_transport.md#iceoryx2-services
auto frame_service(Node& node, const std::string& service) {
    auto name = iox2::ServiceName::create(service.c_str());
    if (!name.has_value()) fail("service name " + service);
    auto result = node.service_builder(name.value())
                      .template publish_subscribe<Payload>()
                      .template user_header<FrameHeader>()
                      .max_publishers(MAX_PUBLISHERS)
                      .max_subscribers(MAX_SUBSCRIBERS)
                      .history_size(HISTORY_SIZE)
                      .subscriber_max_buffer_size(SUBSCRIBER_MAX_BUFFER_SIZE)
                      .enable_safe_overflow(ENABLE_SAFE_OVERFLOW)
                      .open_or_create();
    if (!result.has_value()) fail("service " + service + ": " + text(result.error()));
    return std::move(result.value());
}

auto notify_service(Node& node, const std::string& service) {
    auto name = iox2::ServiceName::create((service + NOTIFY_SUFFIX).c_str());
    if (!name.has_value()) fail("service name " + service);
    auto result = node.service_builder(name.value()).event().open_or_create();
    if (!result.has_value()) fail("notify service " + service + ": " + text(result.error()));
    return std::move(result.value());
}

using FrameServiceT = decltype(frame_service(std::declval<Node&>(), std::string()));
using NotifyServiceT = decltype(notify_service(std::declval<Node&>(), std::string()));
using PublisherT = std::decay_t<decltype(std::declval<FrameServiceT&>().publisher_builder().create().value())>;
using NotifierT = std::decay_t<decltype(std::declval<NotifyServiceT&>().notifier_builder().create().value())>;
using SubscriberT = std::decay_t<decltype(std::declval<FrameServiceT&>().subscriber_builder().create().value())>;
using ListenerT = std::decay_t<decltype(std::declval<NotifyServiceT&>().listener_builder().create().value())>;
using SampleT = std::decay_t<decltype(std::declval<SubscriberT&>().receive().value().value())>;

}

struct FramePublisher::Impl {
    Node node;
    FrameServiceT service;
    NotifyServiceT notify;
    PublisherT publisher;
    NotifierT notifier;
    std::size_t max_len;

    Impl(const std::string& name, std::size_t len)
        : node(make_node()),
          service(frame_service(node, name)),
          notify(notify_service(node, name)),
          publisher(make_publisher(service, len)),
          notifier(make_notifier(notify)),
          max_len(len) {}

    static PublisherT make_publisher(FrameServiceT& s, std::size_t len) {
        auto p = s.publisher_builder()
                     .initial_max_slice_len(len)
                     .allocation_strategy(iox2::AllocationStrategy::PowerOfTwo)
                     .create();
        if (!p.has_value()) fail("publisher: " + text(p.error()));
        return std::move(p.value());
    }
    static NotifierT make_notifier(NotifyServiceT& s) {
        auto n = s.notifier_builder().create();
        if (!n.has_value()) fail("notifier: " + text(n.error()));
        return std::move(n.value());
    }
};

FramePublisher::FramePublisher(const std::string& service, std::size_t initial_max_len)
    : impl_(std::make_unique<Impl>(service, initial_max_len)) {}

FramePublisher::~FramePublisher() = default;

std::size_t FramePublisher::initial_max_len() const { return impl_->max_len; }

// doc/frame_transport.md#iceoryx2-services
void FramePublisher::publish(const TargetScopeFrameHeader& header,
                             const std::function<void(std::span<std::uint8_t>)>& fill) {
    const std::size_t len = static_cast<std::size_t>(header.stride) * header.height;
    auto loaned = impl_->publisher.loan_slice_uninit(len);
    if (!loaned.has_value()) fail("loan: " + text(loaned.error()));
    auto sample = std::move(loaned.value());
    static_cast<TargetScopeFrameHeader&>(sample.user_header_mut()) = header;
    auto slice = sample.payload_mut();
    fill(std::span<std::uint8_t>(slice.data(), slice.number_of_bytes()));
    auto sent = iox2::send(iox2::assume_init(std::move(sample)));
    if (!sent.has_value()) fail("send: " + text(sent.error()));
    auto notified = impl_->notifier.notify();
    if (!notified.has_value()) fail("notify: " + text(notified.error()));
}

struct Frame::Impl {
    SampleT sample;
    explicit Impl(SampleT s) : sample(std::move(s)) {}
};

const TargetScopeFrameHeader& Frame::header() const { return impl_->sample.user_header(); }

std::span<const std::uint8_t> Frame::data() const {
    auto payload = impl_->sample.payload();
    return {payload.data(), payload.number_of_bytes()};
}

struct FrameSubscriber::Impl {
    Node node;
    FrameServiceT service;
    NotifyServiceT notify;
    SubscriberT subscriber;
    ListenerT listener;

    explicit Impl(const std::string& name)
        : node(make_node()),
          service(frame_service(node, name)),
          notify(notify_service(node, name)),
          subscriber(make_subscriber(service)),
          listener(make_listener(notify)) {}

    static SubscriberT make_subscriber(FrameServiceT& s) {
        auto r = s.subscriber_builder().buffer_size(SUBSCRIBER_MAX_BUFFER_SIZE).create();
        if (!r.has_value()) fail("subscriber: " + text(r.error()));
        return std::move(r.value());
    }
    static ListenerT make_listener(NotifyServiceT& s) {
        auto r = s.listener_builder().create();
        if (!r.has_value()) fail("listener: " + text(r.error()));
        return std::move(r.value());
    }

    // doc/frame_transport.md#subscriber-rules
    std::optional<Frame> newest() {
        std::optional<SampleT> newest;
        for (;;) {
            auto received = subscriber.receive();
            if (!received.has_value()) {
                if (newest) break;
                fail("receive: " + text(received.error()));
            }
            if (!received.value().has_value()) break;
            newest = std::move(received.value().value());
        }
        if (!newest) return std::nullopt;
        Frame f;
        f.impl_ = std::make_shared<Frame::Impl>(std::move(*newest));
        return f;
    }
};

FrameSubscriber::FrameSubscriber(const std::string& service) : impl_(std::make_unique<Impl>(service)) {}
FrameSubscriber::~FrameSubscriber() = default;

std::optional<Frame> FrameSubscriber::wait(std::chrono::milliseconds timeout) {
    if (auto f = impl_->newest()) return f;
    auto waited = impl_->listener.timed_wait(
        [](auto) {}, iox2::bb::Duration::from_millis(static_cast<std::uint64_t>(timeout.count())));
    (void)waited;
    return impl_->newest();
}

// doc/frame_transport.md#stale-shared-memory
void cleanup_dead_nodes() {
    auto result = Node::list(iox2::Config().view(), [](iox2::NodeState<iox2::ServiceType::Ipc> state) {
        state.dead([](iox2::DeadNodeView<iox2::ServiceType::Ipc>& view) {
            auto removed = view.try_remove_stale_resources();
            if (!removed.has_value()) spdlog::warn("stale iceoryx2 resources could not be removed");
        });
        return iox2::CallbackProgression::Continue;
    });
    (void)result;
}

}
