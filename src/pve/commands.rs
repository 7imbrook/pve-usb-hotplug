use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Command {
    #[serde(rename = "qmp_capabilities")]
    Handshake,
    #[serde(rename = "qom-list")]
    QomList,
    #[serde(rename = "qom-get")]
    QomGet,
    #[serde(rename = "query-commands")]
    QueryCommands,
    #[serde(rename = "device_add")]
    DeviceAdd,
    #[serde(rename = "device_del")]
    DeviceRemove,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum Argument<'a> {
    Handshake {},
    QueryCommands {},
    QomList {
        path: &'a str,
    },
    QomGet {
        path: &'a str,
        property: &'a str,
    },
    DeviceAdd {
        id: &'a str,
        driver: &'a str,
        bus: &'a str,
        vendorid: &'a str,
        productid: &'a str,
    },
    DeviceRemove {
        id: &'a str,
    },
}

#[derive(Debug, Serialize)]
pub struct QMPMessage<'a> {
    pub execute: Command,
    pub arguments: Argument<'a>,
}

pub fn build_command(args: Argument) -> QMPMessage {
    use Argument::*;
    match args {
        Handshake {} => QMPMessage {
            execute: Command::Handshake,
            arguments: args,
        },
        QomList { .. } => QMPMessage {
            execute: Command::QomList,
            arguments: args,
        },
        QomGet { .. } => QMPMessage {
            execute: Command::QomGet,
            arguments: args,
        },
        QueryCommands { .. } => QMPMessage {
            execute: Command::QueryCommands,
            arguments: args,
        },
        DeviceAdd { .. } => QMPMessage {
            execute: Command::DeviceAdd,
            arguments: args,
        },
        DeviceRemove { .. } => QMPMessage {
            execute: Command::DeviceRemove,
            arguments: args,
        },
    }
}

pub mod response {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct QueryCommand {
        #[serde(rename = "return")]
        pub items: Vec<serde_json::Value>,
    }
    #[derive(Debug, Deserialize)]
    pub struct Item {
        pub name: String,
        #[serde(rename = "type")]
        pub kind: String,
    }
    #[derive(Debug, Deserialize)]
    pub struct QomList {
        #[serde(rename = "return")]
        pub items: Vec<Item>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn test_command_handshake() {
        assert_eq!(
            r#"{"execute":"qmp_capabilities","arguments":{}}"#,
            to_string(&build_command(Argument::Handshake {})).unwrap()
        );
    }

    #[test]
    fn test_command_qom_list() {
        assert_eq!(
            r#"{"execute":"qom-list","arguments":{"path":"/"}}"#,
            to_string(&build_command(Argument::QomList { path: "/" })).unwrap()
        );
    }
    #[test]
    fn test_command_qom_get() {
        assert_eq!(
            r#"{"execute":"qom-get","arguments":{"path":"/","property":"type"}}"#,
            to_string(&build_command(Argument::QomGet {
                path: "/",
                property: "type"
            }))
            .unwrap()
        );
    }
}
