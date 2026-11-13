// doc/programs/launcher.md#startup-sequence

use std::collections::{HashMap, HashSet};

use crate::config::ProgramSpec;

pub fn start_order(programs: &[ProgramSpec]) -> Result<Vec<String>, String> {
    let by_name: HashMap<&str, &ProgramSpec> = programs.iter().map(|p| (p.name.as_str(), p)).collect();
    for p in programs {
        for d in &p.depends_on {
            if !by_name.contains_key(d.as_str()) {
                return Err(format!("{} depends on {d}, which isn't configured", p.name));
            }
        }
    }
    let mut order = Vec::new();
    let mut done = HashSet::new();
    let mut visiting = Vec::new();
    fn visit<'a>(
        name: &'a str,
        by_name: &HashMap<&'a str, &'a ProgramSpec>,
        done: &mut HashSet<&'a str>,
        visiting: &mut Vec<&'a str>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if done.contains(name) {
            return Ok(());
        }
        if let Some(i) = visiting.iter().position(|v| *v == name) {
            let cycle: Vec<&str> = visiting[i..].iter().copied().chain([name]).collect();
            return Err(format!("dependency cycle: {}", cycle.join(" -> ")));
        }
        visiting.push(name);
        for d in &by_name[name].depends_on {
            visit(d, by_name, done, visiting, order)?;
        }
        visiting.pop();
        done.insert(name);
        order.push(name.to_string());
        Ok(())
    }
    for p in programs {
        visit(&p.name, &by_name, &mut done, &mut visiting, &mut order)?;
    }
    Ok(order)
}

// doc/programs/launcher.md#startup-sequence
pub fn enabled_dependencies<'a>(p: &'a ProgramSpec, programs: &'a [ProgramSpec]) -> Vec<&'a str> {
    p.depends_on
        .iter()
        .filter(|d| programs.iter().any(|q| &q.name == *d && q.enabled))
        .map(String::as_str)
        .collect()
}
