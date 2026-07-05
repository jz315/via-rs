use crate::epru::EpruWriter;
use crate::ids::json_escape;

pub(crate) fn render_project2_json(title: &str) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"title\": \"{}\",\n",
            "  \"cbb_project\": false,\n",
            "  \"editorVersion\": \"\",\n",
            "  \"introduction\": \"\",\n",
            "  \"description\": \"\",\n",
            "  \"tags\": \"[]\"\n",
            "}}\n"
        ),
        json_escape(title),
    )
}

pub(crate) fn render_config_document(writer: &mut EpruWriter, _title: &str) {
    writer.dochead("CONFIG", "CONFIG");
    writer.record_with_id("META", "META", "{\"defaultSheet\":\"\"}");
}

pub(crate) fn render_font_document(writer: &mut EpruWriter) {
    writer.dochead("FONT", "FONT");
    writer.record_with_id(
        "FONT",
        "default2",
        "{\"fontFamily\":\"default2\",\"source\":\"system\"}",
    );
}
