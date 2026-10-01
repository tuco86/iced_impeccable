//! The widget inventory: what the interface reports to a widget operation,
//! the target grammar that addresses it (`#ID`, `~SUBSTRING`, exact text),
//! and the text form of nodes and match errors.

use std::fmt;

use iced::advanced::widget;
use iced::{Rectangle, Vector};
use iced_selector::Candidate;

/// What a command addresses.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Target {
    /// `#ID`: a widget id given with `.id("...")`.
    Id(String),
    /// `~SUBSTRING`: text that contains the substring.
    Contains(String),
    /// Anything else: text that is exactly this.
    Text(String),
}

impl Target {
    pub(crate) fn parse(text: &str) -> Self {
        if let Some(id) = text.strip_prefix('#').filter(|id| !id.is_empty()) {
            Self::Id(id.to_owned())
        } else if let Some(sub) = text.strip_prefix('~').filter(|sub| !sub.is_empty()) {
            Self::Contains(sub.to_owned())
        } else {
            Self::Text(text.to_owned())
        }
    }

    pub(crate) fn matches(&self, node: &Node) -> bool {
        match self {
            Self::Id(id) => node.raw_id.as_ref() == Some(&widget::Id::from(id.clone())),
            Self::Contains(sub) => node
                .text
                .as_deref()
                .is_some_and(|t| t.contains(sub.as_str())),
            Self::Text(text) => node.text.as_deref() == Some(text.as_str()),
        }
    }

    /// The text a "similar" hint compares against, if the target is text.
    fn needle(&self) -> Option<&str> {
        match self {
            Self::Id(_) => None,
            Self::Contains(text) | Self::Text(text) => Some(text),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(id) => write!(f, "#{id}"),
            Self::Contains(sub) => write!(f, "~{sub}"),
            Self::Text(text) => f.write_str(text),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Text,
    Input,
    Focusable,
    Scrollable,
    Container,
    Custom,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Input => "input",
            Self::Focusable => "focusable",
            Self::Scrollable => "scrollable",
            Self::Container => "container",
            Self::Custom => "custom",
        }
    }

    /// The outline colour `screenshot --annotate` draws this kind in.
    pub(crate) fn color(self) -> [u8; 4] {
        match self {
            Self::Text => [0, 200, 255, 255],
            Self::Input | Self::Focusable => [255, 200, 0, 255],
            Self::Container => [255, 0, 200, 255],
            Self::Scrollable => [0, 220, 120, 255],
            Self::Custom => [255, 120, 0, 255],
        }
    }
}

/// One widget as the interface reports it to an operation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Node {
    pub(crate) kind: Kind,
    pub(crate) raw_id: Option<widget::Id>,
    /// The name of a `Custom` id; `Unique` ids have none.
    pub(crate) id: Option<String>,
    /// Layout bounds, before any scroll translation.
    pub(crate) bounds: Rectangle,
    /// The on-screen part, `None` when scrolled or clipped away.
    pub(crate) visible: Option<Rectangle>,
    pub(crate) text: Option<String>,
    pub(crate) focused: bool,
    /// Translation and content bounds of a scrollable.
    pub(crate) scroll: Option<(Vector, Rectangle)>,
}

impl Node {
    /// Visible bounds when there are any, layout bounds otherwise.
    pub(crate) fn rect(&self) -> Rectangle {
        self.visible.unwrap_or(self.bounds)
    }
}

/// The selector run over the whole tree: every candidate becomes a node.
pub(crate) fn select(candidate: Candidate<'_>) -> Option<Node> {
    let raw_id = candidate.id().cloned();
    let bounds = candidate.bounds();
    let visible = candidate.visible_bounds();
    let mut node = Node {
        kind: Kind::Custom,
        id: raw_id.as_ref().and_then(id_label),
        raw_id,
        bounds,
        visible,
        text: None,
        focused: false,
        scroll: None,
    };
    match candidate {
        Candidate::Container { .. } => node.kind = Kind::Container,
        Candidate::Focusable { state, .. } => {
            node.kind = Kind::Focusable;
            node.focused = state.is_focused();
        }
        Candidate::Scrollable {
            translation,
            content_bounds,
            ..
        } => {
            node.kind = Kind::Scrollable;
            node.scroll = Some((translation, content_bounds));
        }
        Candidate::TextInput { state, .. } => {
            node.kind = Kind::Input;
            node.text = Some(state.text().to_owned());
        }
        Candidate::Text { content, .. } => {
            node.kind = Kind::Text;
            node.text = Some(content.to_owned());
        }
        Candidate::Custom { .. } => {}
    }
    Some(node)
}

/// The name inside a `Custom` widget id. `widget::Id` has no accessor; its
/// `Debug` form is `Id(Custom("name"))` or `Id(Unique(7))`.
pub(crate) fn id_label(id: &widget::Id) -> Option<String> {
    let debug = format!("{id:?}");
    let quoted = debug.strip_prefix("Id(Custom(")?.strip_suffix("))")?;
    let inner = quoted.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '0' => out.push('\0'),
                other => out.push(other),
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

/// Drops the second report of a widget that reports itself twice
/// (`text_input` is both a text input and a focusable), keeping the focus
/// state of either report.
pub(crate) fn dedupe(nodes: Vec<Node>) -> Vec<Node> {
    let mut kept: Vec<Node> = Vec::with_capacity(nodes.len());
    for node in nodes {
        if let Some(earlier) = kept
            .iter_mut()
            .find(|k| k.raw_id == node.raw_id && k.bounds == node.bounds && k.raw_id.is_some())
        {
            earlier.focused |= node.focused;
            continue;
        }
        kept.push(node);
    }
    kept
}

/// The nodes `tree` and `--annotate` show: anonymous containers only when
/// nothing else lies inside them, so layout rows and columns drop out while
/// an icon-only button stays addressable.
pub(crate) fn filtered(nodes: &[Node], all: bool) -> Vec<&Node> {
    if all {
        return nodes.iter().collect();
    }
    nodes
        .iter()
        .enumerate()
        .filter(|(i, node)| {
            node.kind != Kind::Container
                || node.id.is_some()
                || !nodes
                    .iter()
                    .enumerate()
                    .any(|(j, other)| j != *i && inside(other.bounds, node.bounds))
        })
        .map(|(_, node)| node)
        .collect()
}

fn inside(inner: Rectangle, outer: Rectangle) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

/// `X Y W H`, rounded to logical pixels.
pub(crate) fn rect_words(r: Rectangle) -> String {
    format!(
        "{} {} {} {}",
        r.x.round(),
        r.y.round(),
        r.width.round(),
        r.height.round()
    )
}

/// The point `tap` clicks: the centre of the visible bounds, rounded.
pub(crate) fn centre(r: Rectangle) -> (f32, f32) {
    (
        (r.x + r.width / 2.0).round(),
        (r.y + r.height / 2.0).round(),
    )
}

/// `{idx} {kind} at {cx},{cy} box {x} {y} {w} {h}[ id=..][ focused][ hidden][ offset ..][ text=".."]`.
pub(crate) fn node_line(index: usize, node: &Node) -> String {
    let rect = node.rect();
    let (cx, cy) = centre(rect);
    let mut line = format!(
        "{index} {} at {cx},{cy} box {}",
        node.kind.name(),
        rect_words(rect)
    );
    if let Some(id) = &node.id {
        line.push_str(&format!(" id={}", escape(id)));
    }
    if node.focused {
        line.push_str(" focused");
    }
    if node.visible.is_none() {
        line.push_str(" hidden");
    }
    if let Some((offset, content)) = node.scroll {
        line.push_str(&format!(
            " offset {},{} content {}x{}",
            offset.x.round(),
            offset.y.round(),
            content.width.round(),
            content.height.round()
        ));
    }
    if let Some(text) = &node.text {
        line.push_str(&format!(" text=\"{}\"", escape(text)));
    }
    line
}

/// Text made safe for one protocol line: `\` and `"` escaped, control and
/// Private Use Area characters (icon fonts) as `\u{XXXX}`, cut after 80
/// characters.
pub(crate) fn escape(text: &str) -> String {
    const LIMIT: usize = 80;
    let mut out = String::new();
    for (i, c) in text.chars().enumerate() {
        if i == LIMIT {
            out.push_str("...");
            break;
        }
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c if c.is_control()
                || ('\u{E000}'..='\u{F8FF}').contains(&c)
                || ('\u{F0000}'..='\u{10FFFF}').contains(&c) =>
            {
                out.push_str(&format!("\\u{{{:04X}}}", u32::from(c)));
            }
            c => out.push(c),
        }
    }
    out
}

/// Resolves a target to one node for `find` and `tap`, or the error reply
/// without its `err ` prefix.
pub(crate) fn pick<'a>(
    nodes: &'a [Node],
    target: &Target,
    nth: Option<usize>,
) -> Result<&'a Node, String> {
    let matches: Vec<&Node> = nodes.iter().filter(|n| target.matches(n)).collect();
    let chosen = match (nth, matches.len()) {
        (_, 0) => return Err(not_found(nodes, target)),
        (Some(n), len) => *matches
            .get(n.wrapping_sub(1))
            .ok_or_else(|| format!("only {len} matches for {target}"))?,
        (None, 1) => matches[0],
        (None, n) => {
            let listed: Vec<String> = matches
                .iter()
                .take(5)
                .map(|m| rect_words(m.rect()))
                .collect();
            return Err(format!(
                "ambiguous: {n} matches (use --nth): {}",
                listed.join("; ")
            ));
        }
    };
    if chosen.visible.is_none() {
        return Err(format!(
            "not visible: {} (scroll it into view)",
            rect_words(chosen.bounds)
        ));
    }
    Ok(chosen)
}

/// `not found: TARGET` with up to three texts that contain the target's
/// text or are contained in it, ignoring case.
pub(crate) fn not_found(nodes: &[Node], target: &Target) -> String {
    let mut message = format!("not found: {target}");
    let Some(needle) = target.needle().map(str::to_lowercase) else {
        return message;
    };
    let mut similar: Vec<&str> = Vec::new();
    for text in nodes.iter().filter_map(|n| n.text.as_deref()) {
        let lower = text.to_lowercase();
        if text.is_empty() || similar.contains(&text) {
            continue;
        }
        if lower.contains(&needle) || needle.contains(&lower) {
            similar.push(text);
            if similar.len() == 3 {
                break;
            }
        }
    }
    if !similar.is_empty() {
        let quoted: Vec<String> = similar
            .iter()
            .map(|t| format!("\"{}\"", escape(t)))
            .collect();
        message.push_str(&format!("; similar: {}", quoted.join(", ")));
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(
        kind: Kind,
        id: Option<&'static str>,
        rect: (f32, f32, f32, f32),
        text: Option<&str>,
    ) -> Node {
        let bounds = Rectangle::new(
            iced::Point::new(rect.0, rect.1),
            iced::Size::new(rect.2, rect.3),
        );
        Node {
            kind,
            raw_id: id.map(widget::Id::new),
            id: id.map(str::to_owned),
            bounds,
            visible: Some(bounds),
            text: text.map(str::to_owned),
            focused: false,
            scroll: None,
        }
    }

    #[test]
    fn id_label_reads_custom_and_skips_unique() {
        assert_eq!(id_label(&widget::Id::new("name")).as_deref(), Some("name"));
        assert_eq!(
            id_label(&widget::Id::from("a \"b\"\\c".to_owned())).as_deref(),
            Some("a \"b\"\\c")
        );
        assert_eq!(id_label(&widget::Id::unique()), None);
    }

    #[test]
    fn target_grammar() {
        assert_eq!(Target::parse("#name"), Target::Id("name".into()));
        assert_eq!(Target::parse("~Sav"), Target::Contains("Sav".into()));
        assert_eq!(Target::parse("Row 60"), Target::Text("Row 60".into()));
        // A bare sigil is text, not an empty id.
        assert_eq!(Target::parse("#"), Target::Text("#".into()));
        assert_eq!(Target::parse("~"), Target::Text("~".into()));
    }

    #[test]
    fn id_target_matches_custom_ids_only() {
        let n = node(
            Kind::Container,
            Some("settings"),
            (0.0, 0.0, 10.0, 10.0),
            None,
        );
        assert!(Target::parse("#settings").matches(&n));
        assert!(!Target::parse("#other").matches(&n));
        assert!(!Target::parse("settings").matches(&n));
    }

    #[test]
    fn pick_reports_ambiguity_nth_and_visibility() {
        let mut nodes = vec![
            node(Kind::Text, None, (0.0, 0.0, 40.0, 20.0), Some("Save")),
            node(Kind::Text, None, (0.0, 30.0, 40.0, 20.0), Some("Save")),
            node(Kind::Text, None, (0.0, 900.0, 40.0, 20.0), Some("Row 60")),
        ];
        nodes[2].visible = None;
        let save = Target::parse("Save");
        assert_eq!(
            pick(&nodes, &save, None).unwrap_err(),
            "ambiguous: 2 matches (use --nth): 0 0 40 20; 0 30 40 20"
        );
        assert_eq!(pick(&nodes, &save, Some(2)).unwrap().bounds.y, 30.0);
        assert_eq!(
            pick(&nodes, &save, Some(3)).unwrap_err(),
            "only 2 matches for Save"
        );
        assert_eq!(
            pick(&nodes, &Target::parse("Row 60"), None).unwrap_err(),
            "not visible: 0 900 40 20 (scroll it into view)"
        );
        assert_eq!(
            pick(&nodes, &Target::parse("sav"), None).unwrap_err(),
            "not found: sav; similar: \"Save\""
        );
    }

    #[test]
    fn dedupe_merges_double_reports_and_keeps_focus() {
        let mut focusable = node(Kind::Focusable, Some("name"), (0.0, 0.0, 100.0, 30.0), None);
        focusable.focused = true;
        let nodes = vec![
            node(
                Kind::Input,
                Some("name"),
                (0.0, 0.0, 100.0, 30.0),
                Some("Ada"),
            ),
            focusable,
        ];
        let nodes = dedupe(nodes);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].kind, Kind::Input);
        assert!(nodes[0].focused);
    }

    #[test]
    fn filter_drops_anonymous_containers_with_content() {
        let nodes = vec![
            node(Kind::Container, None, (0.0, 0.0, 100.0, 100.0), None),
            node(Kind::Text, None, (10.0, 10.0, 20.0, 10.0), Some("Save")),
            // An icon-only button: nothing inside, so it stays.
            node(Kind::Container, None, (50.0, 50.0, 16.0, 16.0), None),
            node(
                Kind::Container,
                Some("settings"),
                (0.0, 0.0, 100.0, 100.0),
                None,
            ),
        ];
        let shown: Vec<_> = filtered(&nodes, false)
            .into_iter()
            .map(|n| (n.kind, n.bounds.x))
            .collect();
        assert_eq!(
            shown,
            [
                (Kind::Text, 10.0),
                (Kind::Container, 50.0),
                (Kind::Container, 0.0)
            ]
        );
        assert_eq!(filtered(&nodes, true).len(), 4);
    }

    #[test]
    fn node_line_format_and_escaping() {
        let mut n = node(
            Kind::Input,
            Some("name"),
            (10.0, 20.0, 100.0, 30.0),
            Some("a\"b\\\u{E001}\n"),
        );
        n.focused = true;
        assert_eq!(
            node_line(3, &n),
            "3 input at 60,35 box 10 20 100 30 id=name focused text=\"a\\\"b\\\\\\u{E001}\\u{000A}\""
        );
        let long = "x".repeat(100);
        assert_eq!(escape(&long), format!("{}...", "x".repeat(80)));
    }
}
