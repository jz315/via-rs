use crate::constants::{CLIENT_ID, EDIT_VERSION, EXPORT_TIME_MS, USER_NAME, USER_UUID};
use crate::ids::json_escape;
use crate::units::opt_i32;

pub(crate) struct SymbolAttr<'a> {
    pub(crate) id: String,
    pub(crate) part_id: Option<String>,
    pub(crate) parent_id: &'a str,
    pub(crate) key: &'a str,
    pub(crate) value: &'a str,
    pub(crate) x: Option<i32>,
    pub(crate) y: Option<i32>,
    pub(crate) visible: bool,
    pub(crate) z_index: usize,
}

pub(crate) struct EpruWriter {
    out: String,
    dochead_ticket: usize,
    ticket: usize,
}

impl EpruWriter {
    pub(crate) fn new() -> Self {
        Self {
            out: String::new(),
            dochead_ticket: 1,
            ticket: 1,
        }
    }

    pub(crate) fn finish(self) -> String {
        self.out
    }

    pub(crate) fn dochead(&mut self, doc_type: &str, uuid: &str) {
        self.out.push_str("{\"type\":\"DOCHEAD\",\"ticket\":");
        self.out.push_str(&self.dochead_ticket.to_string());
        self.out.push_str("}||");
        self.out.push_str(&format!(
            concat!(
                "{{\"docType\":\"{}\",\"client\":\"{}\",\"uuid\":\"{}\",",
                "\"updateTime\":{},\"version\":\"{}\",\"editVersion\":\"{}\",",
                "\"user\":{{\"uuid\":\"{}\",\"nickname\":\"{}\",\"username\":\"{}\",\"avatar\":\"\"}}}}"
            ),
            json_escape(doc_type),
            CLIENT_ID,
            json_escape(uuid),
            EXPORT_TIME_MS,
            EXPORT_TIME_MS,
            EDIT_VERSION,
            USER_UUID,
            USER_NAME,
            USER_NAME,
        ));
        self.out.push_str("|\n");
        self.dochead_ticket += 1;
        self.ticket = 1;
    }

    pub(crate) fn record_with_id(&mut self, kind: &str, id: &str, body: &str) {
        self.record(kind, Some(id), body);
    }

    pub(crate) fn record(&mut self, kind: &str, id: Option<&str>, body: &str) {
        self.out.push_str("{\"type\":\"");
        self.out.push_str(&json_escape(kind));
        self.out.push_str("\",\"ticket\":");
        self.out.push_str(&self.ticket.to_string());
        if let Some(id) = id {
            self.out.push_str(",\"id\":\"");
            self.out.push_str(&json_escape(id));
            self.out.push('"');
        }
        self.out.push_str("}||");
        self.out.push_str(body);
        self.out.push_str("|\n");
        self.ticket += 1;
    }

    pub(crate) fn attr(&mut self, attr: SymbolAttr<'_>) {
        self.record_with_id(
            "ATTR",
            &attr.id,
            &format!(
                concat!(
                    "{{{}\"groupId\":\"\",\"locked\":false,\"zIndex\":{},",
                    "\"parentId\":\"{}\",\"key\":\"{}\",\"value\":\"{}\",",
                    "\"keyVisible\":false,\"valueVisible\":{},",
                    "\"x\":{},\"y\":{},",
                    "\"rotation\":0,\"color\":null,\"fillColor\":null,\"fontFamily\":null,",
                    "\"fontSize\":null,\"strikeout\":false,\"underline\":false,",
                    "\"italic\":false,\"fontWeight\":false,\"align\":\"LEFT_BOTTOM\",",
                    "\"version\":\"2.0\"}}"
                ),
                attr.part_id
                    .as_ref()
                    .map(|part_id| format!("\"partId\":\"{}\",", json_escape(part_id)))
                    .unwrap_or_default(),
                attr.z_index,
                json_escape(attr.parent_id),
                json_escape(attr.key),
                json_escape(attr.value),
                attr.visible,
                opt_i32(attr.x),
                opt_i32(attr.y),
            ),
        );
    }

    pub(crate) fn wire(&mut self, id: &str, z_index: usize, net_name: &str, points: &[(i32, i32)]) {
        self.record_with_id(
            "WIRE",
            id,
            &format!(
                "{{\"groupId\":\"\",\"locked\":false,\"zIndex\":{}}}",
                z_index,
            ),
        );
        for (index, segment) in points.windows(2).enumerate() {
            let [(start_x, start_y), (end_x, end_y)] = segment else {
                continue;
            };
            self.record_with_id(
                "LINE",
                &format!("{id}_line_{index}"),
                &format!(
                    concat!(
                        "{{\"lineGroup\":\"{}\",\"startX\":{},\"startY\":{},",
                        "\"endX\":{},\"endY\":{},\"strokeColor\":null,",
                        "\"strokeStyle\":null,\"fillColor\":\"none\",",
                        "\"strokeWidth\":null,\"fillStyle\":null}}"
                    ),
                    json_escape(id),
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                ),
            );
        }
        self.record_with_id(
            "ATTR",
            &format!("{id}_net"),
            &format!(
                concat!(
                    "{{\"partId\":\"\",\"groupId\":\"\",\"locked\":true,\"zIndex\":{},",
                    "\"parentId\":\"{}\",\"key\":\"NET\",\"value\":\"{}\",",
                    "\"keyVisible\":false,\"valueVisible\":false,",
                    "\"x\":null,\"y\":null,\"rotation\":0,",
                    "\"color\":null,\"fillColor\":null,\"fontFamily\":null,\"fontSize\":null,",
                    "\"strikeout\":null,\"underline\":null,\"italic\":null,\"fontWeight\":null,",
                    "\"align\":null,\"version\":\"2.0\"}}"
                ),
                z_index + 1,
                json_escape(id),
                json_escape(net_name),
            ),
        );
    }
}
