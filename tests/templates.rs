use rudis_lite::templates;
use std::collections::HashSet;

#[test]
fn catalog_is_complete_and_deterministic() {
    let expected = [
        "analyze",
        "brd",
        "checklist",
        "clarify",
        "constitution",
        "converge",
        "deploy",
        "fan",
        "implement",
        "operate",
        "plan",
        "postmortem",
        "prototype",
        "redesign",
        "revamp",
        "rudis",
        "specify",
        "taskstoissues",
        "tasks",
        "test",
        "tracer",
        "ultimate",
        "upgrade",
    ];
    let actual: Vec<_> = templates::all()
        .iter()
        .map(|template| template.name)
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), actual.iter().collect::<HashSet<_>>().len());
    assert!(templates::all()
        .iter()
        .all(|template| !template.content.is_empty()));
    assert!(!templates::UPSTREAM_REVISION.contains("TODO"));
}

#[test]
fn finds_a_template_by_name() {
    assert_eq!(
        templates::find("specify").unwrap().source,
        "templates/commands/specify.md"
    );
    assert!(templates::find("missing").is_none());
}

#[test]
fn managed_content_keeps_claude_frontmatter_first() {
    let content = templates::managed_content(templates::find("specify").unwrap());
    assert!(content.starts_with("---\n"));
    assert!(content.contains(templates::OWNERSHIP_MARKER));
    assert!(content.find(templates::OWNERSHIP_MARKER).unwrap() > content.find("---\n").unwrap());
}
