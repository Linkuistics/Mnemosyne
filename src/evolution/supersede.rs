use crate::knowledge::entry::Entry;
use chrono::NaiveDate;

pub fn supersede_content(
    entry: &mut Entry,
    old_content_summary: &str,
    reason: &str,
    supersede_date: NaiveDate,
) {
    let created_date = entry.created;
    let superseded_section = format!(
        "\n\n## Superseded\n\n### {} ({} → {})\n> {}\n\n**Reason superseded:** {}\n",
        old_content_summary, created_date, supersede_date, old_content_summary, reason
    );

    if entry.body.contains("## Superseded") {
        entry.body.push_str(&format!(
            "\n### {} ({} → {})\n> {}\n\n**Reason superseded:** {}\n",
            old_content_summary, created_date, supersede_date, old_content_summary, reason
        ));
    } else {
        entry.body.push_str(&superseded_section);
    }
}
