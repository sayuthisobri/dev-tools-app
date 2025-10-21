#![allow(dead_code)]
use crate::utils::{get_parent_path, read_file};
use anyhow::Result;
use resolve_path::PathResolveExt;
use roxmltree::{Document, ExpandedName, Node};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub fields: Vec<Field>,
}

impl Field {
    pub fn new(name: String) -> Self {
        Field {
            name,
            attributes: HashMap::new(),
            fields: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: String,
    pub address: String,
    pub binding: Binding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub input: Field,
    pub output: Field,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub name: String,
    pub transport: String,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wsdl {
    pub name: String,
    pub file_path: String,
    pub target_ns: String,
    pub services: HashMap<String, Vec<ServicePort>>,
    pub ns: Vec<String>,
}

impl Wsdl {
    pub(crate) fn load(path: &str) -> Result<Self> {
        let parent_dir = get_parent_path(path).expect("Failed to get parent path");
        let wsdl_raw = read_file(path);
        let opt = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };
        let doc = Document::parse_with_options(&wsdl_raw, opt)
            .expect(format!("Failed to load wsdl {}", path).as_str());
        let root = doc.root_element();
        let target_ns = root.attribute("targetNamespace").unwrap_or("");

        // imports
        let imports = root
            .descendants()
            .filter(|n| {
                match_tag(n.tag_name(), "import", None)
                    && n.has_attribute("schemaLocation")
                    && match_attr(n, "namespace", target_ns)
            })
            .filter_map(|i| i.attribute("schemaLocation"))
            .map(|p| read_file(p.resolve_in(parent_dir)))
            .collect::<Vec<String>>();

        let mut imported_docs: Vec<Document> = Vec::new();
        for content in &imports {
            imported_docs.push(
                Document::parse_with_options(
                    content,
                    roxmltree::ParsingOptions {
                        allow_dtd: true,
                        ..roxmltree::ParsingOptions::default()
                    },
                )
                .expect(format!("Failed to load wsdl {}", path).as_str()),
            );
        }

        // dbg!(imported_docs.last().unwrap().root_element().children().filter(|n| match_tag(n.tag_name(), "element", None))
        //     .map(|n| n.attribute("name").unwrap_or_default().to_string())
        //     .collect::<Vec<_>>());

        println!("imports count: {:?}", imports.len());

        let wsdl = Wsdl {
            name: root.attribute("name").unwrap_or_default().to_string(),
            file_path: path.to_string(),
            target_ns: target_ns.to_string(),
            services: root
                .children()
                .filter(|n| n.is_element() && n.tag_name().name() == "service")
                .fold(HashMap::new(), |mut m, n| {
                    let name = n
                        .attribute("name")
                        .expect("Failed to get service name")
                        .to_string();
                    m.insert(name, prepare_svc_port(&root, &n, &imported_docs));
                    m
                }),
            ns: root
                .namespaces()
                .map(|n| {
                    n.name()
                        .map(|_n| format!("{}:{}", _n, n.uri()))
                        .unwrap_or(n.uri().to_string())
                })
                .collect(),
        };
        Ok(wsdl)
    }
}

fn match_tag(subject: ExpandedName, name: &str, ns: Option<&str>) -> bool {
    subject.name() == name && ns.is_none_or(|n| subject.namespace() == Some(n))
}

fn get_root<'a, 'input: 'a>(subject: &'a Node<'a, 'input>) -> Node<'a, 'input> {
    subject.document().root_element()
}

fn match_attr(subject: &Node, attr: &str, value: &str) -> bool {
    subject.has_attribute(attr) && subject.attribute(attr) == Some(value)
}

fn match_name(subject: &Node, name: &str) -> bool {
    let mut local_name = name;
    if let Some((prefix, name)) = name.rsplit_once(':') {
        if !subject
            .document()
            .root_element()
            .namespaces()
            .any(|n| n.name().map(|_n| _n.to_string()) == Some(prefix.to_string()))
        {
            return false;
        }
        local_name = name;
    }
    subject.attribute("name") == Some(local_name)
}

fn prepare_svc_port(root: &Node, svc: &Node, imported_docs: &Vec<Document>) -> Vec<ServicePort> {
    svc.children()
        .filter(|n| match_tag(n.tag_name(), "port", None) && n.has_attribute("binding"))
        .map(|p| ServicePort {
            name: p.attribute("name").unwrap_or_default().to_string(),
            address: p
                .children()
                .find(|n| match_tag(n.tag_name(), "address", None))
                .map(|a| a.attribute("location").unwrap_or_default().to_string())
                .expect("Failed to get address"),
            binding: prepare_binding(
                root,
                &p,
                p.attribute("binding").unwrap_or_default(),
                imported_docs,
            ),
        })
        .collect()
}

fn prepare_binding(root: &Node, port: &Node, name: &str, imported_docs: &Vec<Document>) -> Binding {
    let binding_iter = root.children().filter(|n| {
        match_tag(
            n.tag_name(),
            "binding",
            Some("http://schemas.xmlsoap.org/wsdl/"),
        ) && match_name(n, name)
    });

    let (transport, operations) = binding_iter
        .map(|b| {
            let port_type = b
                .attribute("type")
                .map(|t| {
                    root.children().find(|n| {
                        match_tag(
                            n.tag_name(),
                            "portType",
                            Some("http://schemas.xmlsoap.org/wsdl/"),
                        ) && match_name(n, t)
                    })
                })
                .flatten()
                .expect("Failed to get port type");
            (b, port_type)
        })
        .fold(("", Vec::new()), |_, (b, p)| {
            let transport = b
                .descendants()
                .find(|n| {
                    match_tag(
                        n.tag_name(),
                        "binding",
                        Some("http://schemas.xmlsoap.org/wsdl/soap/"),
                    )
                })
                .map(|b| {
                    b.attribute("transport")
                        .expect("Failed to get transport info")
                })
                .unwrap_or("");

            (
                transport,
                p.descendants()
                    .filter(|n| {
                        match_tag(
                            n.tag_name(),
                            "operation",
                            Some("http://schemas.xmlsoap.org/wsdl/"),
                        )
                    })
                    .map(|o| Operation {
                        name: o.attribute("name").unwrap_or_default().to_string(),
                        input: prepare_message(&o, "input", &imported_docs),
                        output: prepare_message(&o, "output", &imported_docs),
                    })
                    .collect(),
            )
        });
    Binding {
        name: port.attribute("binding").unwrap_or_default().to_string(),
        transport: transport.to_string(),
        operations,
    }
}

fn prepare_message(operation: &Node, msg_type: &str, imported_docs: &Vec<Document>) -> Field {
    let get_io_node = || -> Node {
        find_child_tag(
            operation,
            msg_type,
            None,
            Some("http://schemas.xmlsoap.org/wsdl/"),
            true,
        )
        .expect("Failed to get input/output node")
    };

    let handle_message_part = |m: Node| -> Field {
        find_child_tag(
            &m,
            "part",
            None,
            Some("http://schemas.xmlsoap.org/wsdl/"),
            false,
        )
        .as_ref()
        .map(|p| prepare_field(p, imported_docs))
        .expect("Failed to get message part")
    };

    let io_node = get_io_node();
    let msg_name = io_node.attribute("message").unwrap_or_default();
    let get_msg_node = |m: &Node| {
        match_tag(
            m.tag_name(),
            "message",
            Some("http://schemas.xmlsoap.org/wsdl/"),
        ) && match_name(m, msg_name)
    };
    get_root(&io_node)
        .children()
        .find(get_msg_node)
        .map(|m| handle_message_part(m))
        .expect("Failed to find message node")
}

fn find_child_tag<'a, 'input: 'a>(
    node: &Node<'a, 'input>,
    tag: &str,
    name: Option<&str>,
    ns: Option<&str>,
    nested: bool,
) -> Option<Node<'a, 'input>> {
    let items: Box<dyn Iterator<Item = Node<'a, 'input>>>;
    if nested {
        items = Box::new(node.descendants())
    } else {
        items = Box::new(node.children())
    }
    items
        .filter(|n| name.is_none_or(|name| match_name(n, name)))
        .find(|p| match_tag(p.tag_name(), tag, ns))
}

fn prepare_field<'a, 'input: 'a>(
    part: &'a Node<'a, 'input>,
    imported_docs: &'a Vec<Document<'input>>,
) -> Field {
    let element_name = part
        .attribute("element")
        .expect("Missing element attribute on part tag");
    let root = get_root(&part);
    let find_element =
        |root: &Node<'a, 'input>| find_child_tag(root, "element", Some(element_name), None, true);
    let element = find_element(&root);
    let element: Option<Node> = if element.is_some() {
        element
    } else if imported_docs.len() > 0 {
        imported_docs
            .iter()
            .map(|d| d.root_element())
            .filter_map(|root| find_element(&root))
            .next()
    } else {
        None
    };

    fn populate_field(n: &Node) -> Field {
        let mut f = Field::new(n.attribute("name").unwrap_or_default().to_string());
        f.attributes = n
            .attributes()
            .filter(|a| !["name"].contains(&a.name()))
            .map(|a| {
                let val = a.value().rsplit_once(':').map(|v| v.1).unwrap_or(a.value());
                (a.name().to_string(), val.to_string())
            })
            .collect();

        let mut current_parent = Some(*n);
        while current_parent.is_some() && current_parent.unwrap().has_children() {
            let parent = current_parent.unwrap();
            // println!("Current parent {:?}", parent.tag_name());
            let elements: Vec<Field> = parent
                .children()
                .filter(|c| match_tag(c.tag_name(), "element", None))
                .map(|c| populate_field(&c))
                .collect();
            if elements.len() > 0 {
                f.fields = elements;
                current_parent = None;
            } else {
                let new_parent = parent
                    .children()
                    .find(|c| c.is_element())
                    .expect("Failed to get first child");
                // println!("New parent: {:?}", new_parent);
                current_parent = Some(new_parent);
            }
        }

        f
    }

    element
        .map(|e| populate_field(&e))
        .expect(format!("Failed to find element {}", element_name).as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wsdl() -> Result<()> {
        let wsdl = dbg!(Wsdl::load("/Users/msms/Library/CloudStorage/OneDrive-ReldynTechSdnBhd/CDX_Shared/Requirement/AML_New/AML-WSDL-KYC-CRP/AMLWS.wsdl"));
        // wsdl.parse()?;
        // dbg!(wsdl);
        if let Err(e) = wsdl {
            dbg!(e);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_logic() -> Result<()> {
        dbg!(match_tag(ExpandedName::from("msms"), "msms", None));
        dbg!(match_tag(ExpandedName::from(("tx", "msms")), "msms", None));
        dbg!(match_tag(
            ExpandedName::from(("tx", "msms")),
            "msms",
            Some("tx")
        ));
        dbg!(match_tag(
            ExpandedName::from(("tx", "msms")),
            "msms",
            Some("txs")
        ));
        Ok(())
    }
}
