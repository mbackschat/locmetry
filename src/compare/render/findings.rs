//! The Findings section of human comparisons, with presence findings bounded
//! as docs/proposals/archive/comparison-presence-summary.md decided: a restructure
//! changes presence for every folder and package under a moved subtree, so a
//! folder or package folds into the outermost changed member or folder that
//! holds it, and a side with more than one line beyond `PRESENCE_LINES`
//! lists that many and counts the rest.
//! JSON keeps every finding.
use super::{label, side_name, Entity, Finding, FindingContext, Side};
use crate::output::{self, counted};
use crate::style::{Role, Theme};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

/// Presence findings listed per side before the rest are counted. A closing
/// line replaces two or more findings; replacing one would hide its name and
/// save no line, as the Immich showcase replay showed on 2026-10-22.
const PRESENCE_LINES: usize = 10;

/// A member or folder whose presence changed, which can hold other entities.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Holder<'a> {
    Member(&'a str),
    Folder(&'a str),
}

impl<'a> Holder<'a> {
    fn of(entity: &'a Entity) -> Option<Self> {
        match entity {
            Entity::Member { name } => Some(Self::Member(name)),
            Entity::Folder { path } => Some(Self::Folder(path)),
            Entity::Package { .. } => None,
        }
    }
}

/// Entities by kind, in the order sentences name them.
#[derive(Default)]
struct Tally {
    members: u64,
    folders: u64,
    packages: u64,
}

impl Tally {
    fn add(&mut self, entity: &Entity) {
        match entity {
            Entity::Member { .. } => self.members += 1,
            Entity::Folder { .. } => self.folders += 1,
            Entity::Package { .. } => self.packages += 1,
        }
    }
    fn merge(&mut self, other: &Tally) {
        self.members += other.members;
        self.folders += other.folders;
        self.packages += other.packages;
    }
    fn total(&self) -> u64 {
        self.members + self.folders + self.packages
    }
    /// The nonzero kinds as words, such as `12 folders and 3 packages`, with
    /// `more` before the first noun when `more` is set.
    fn phrase(&self, more: bool) -> String {
        let kinds = [
            (self.members, "member"),
            (self.folders, "folder"),
            (self.packages, "package"),
        ];
        let words = kinds
            .into_iter()
            .filter(|(count, _)| *count > 0)
            .enumerate()
            .map(|(index, (count, noun))| match more && index == 0 {
                true => counted(count, &format!("more {noun}")),
                false => counted(count, noun),
            });
        and_list(words.collect())
    }
}

/// `a`, `a and b`, or `a, b and c`.
fn and_list(items: Vec<String>) -> String {
    match items.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}

/// The outermost member or folder with a presence finding on the same side
/// that contains `entity`. A folder's own package counts as inside it.
fn holder<'a>(
    entity: &'a Entity,
    context: &'a FindingContext,
    holders: &BTreeSet<(Side, Holder<'a>)>,
) -> Option<Holder<'a>> {
    let held = |holder| holders.contains(&(context.side, holder));
    let (path, own) = match entity {
        Entity::Member { .. } => return None,
        Entity::Folder { path } => (path.as_str(), false),
        Entity::Package { path, .. } => (path.as_str(), true),
    };
    if let Some(member) = context.member.as_deref().map(Holder::Member) {
        if held(member) {
            return Some(member);
        }
    }
    path.match_indices('/')
        .map(|(end, _)| &path[..end])
        .chain(own.then_some(path))
        .map(Holder::Folder)
        .find(|folder| held(*folder))
}

fn presence(finding: &Finding) -> Option<(&Entity, &FindingContext)> {
    match finding {
        Finding::PresenceChanged {
            entity, context, ..
        } => Some((entity, context)),
        _ => None,
    }
}

/// The indexes of presence findings held by another, and what each holder holds.
type Folded<'a> = (BTreeSet<usize>, BTreeMap<(Side, Holder<'a>), Tally>);

fn fold<'a>(findings: &[&'a Finding]) -> Folded<'a> {
    let holders: BTreeSet<_> = findings
        .iter()
        .filter_map(|f| presence(f))
        .filter_map(|(entity, context)| Some((context.side, Holder::of(entity)?)))
        .collect();
    let mut held = BTreeSet::new();
    let mut inside: BTreeMap<_, Tally> = BTreeMap::new();
    for (index, f) in findings.iter().enumerate() {
        let Some((entity, context)) = presence(f) else {
            continue;
        };
        if let Some(holder) = holder(entity, context, &holders) {
            inside
                .entry((context.side, holder))
                .or_default()
                .add(entity);
            held.insert(index);
        }
    }
    (held, inside)
}

fn write_line(out: &mut String, first: &mut bool, code: &str, context: &str, message: &str) {
    let separator = if std::mem::take(first) { "" } else { "\n" };
    writeln!(out, "{separator}- {code} ({context}): {message}").unwrap();
}

pub(super) fn write_findings(
    out: &mut String,
    findings: &[&Finding],
    markdown: bool,
    theme: Theme,
) {
    if findings.is_empty() {
        return;
    }
    match markdown {
        true => write!(out, "\n## Findings\n\n"),
        false => write!(out, "\n{}", output::section_title("Findings", theme)),
    }
    .unwrap();
    let (held, inside) = fold(findings);
    let code = |f: &Finding| theme.paint(Role::Information, f.code());
    let mut lines: BTreeMap<Side, usize> = BTreeMap::new();
    for (index, f) in findings.iter().enumerate() {
        if !held.contains(&index) && presence(f).is_some() {
            *lines.entry(f.context().side).or_default() += 1;
        }
    }
    let limit = |side| match lines[&side] > PRESENCE_LINES + 1 {
        true => PRESENCE_LINES,
        false => usize::MAX,
    };
    let mut listed: BTreeMap<Side, usize> = BTreeMap::new();
    let mut rest: BTreeMap<Side, Tally> = BTreeMap::new();
    let mut first = true;
    for (index, f) in findings.iter().enumerate() {
        if held.contains(&index) {
            continue;
        }
        let context = f.context();
        let mut message = context.message.clone();
        if let Finding::PresenceChanged {
            entity, presence, ..
        } = f
        {
            let contents = Holder::of(entity).and_then(|h| inside.get(&(context.side, h)));
            let count = listed.entry(context.side).or_default();
            *count += 1;
            if *count > limit(context.side) {
                let tally = rest.entry(context.side).or_default();
                tally.add(entity);
                if let Some(contents) = contents {
                    tally.merge(contents);
                }
                continue;
            }
            if let Some(contents) = contents {
                let inside = format!(", with {} inside it", contents.phrase(false));
                message = entity.presence_message(*presence, &inside);
            }
        }
        let member = context
            .member
            .as_ref()
            .map_or(String::new(), |m| format!(", {}", label(m, markdown)));
        let side = format!("{}{member}", side_name(context.side));
        write_line(out, &mut first, &code(f), &side, &label(&message, markdown));
    }
    for (side, tally) in rest {
        let verb = if tally.total() == 1 {
            "appears"
        } else {
            "appear"
        };
        let message = format!(
            "{} {verb} only in the {} report. '--format json' lists each one.",
            tally.phrase(true),
            side_name(side)
        );
        let code = theme.paint(Role::Information, "presence_changed");
        write_line(
            out,
            &mut first,
            &code,
            side_name(side),
            &label(&message, markdown),
        );
    }
}
