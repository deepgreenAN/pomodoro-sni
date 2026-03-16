use std::collections::HashMap;

use indexmap::IndexMap;
use zbus::zvariant::{OwnedValue, Value};

/// メニューアイテムのプロパティディクショナリ．
#[derive(Debug, Default)]
pub struct MenuProperties {
    properties: HashMap<String, OwnedValue>,
}

impl MenuProperties {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }
    /// プロパティディクショナリへの挿入．既に存在する場合は更新する．
    pub fn insert(&mut self, key: String, owned_value: OwnedValue) -> Option<OwnedValue> {
        self.properties.insert(key, owned_value)
    }
    /// rust型のプロパティディクショナリへの挿入．既に存在する場合は更新する．
    pub fn insert_value<'a, T: Into<Value<'a>>>(
        &mut self,
        key: String,
        raw_value: T,
    ) -> Option<OwnedValue> {
        let value: Value<'a> = raw_value.into();

        if let Value::Str(_) | Value::Bool(_) | Value::I32(_) = value {
            self.insert(key, value.try_into_owned().unwrap())
        } else {
            panic!("Properties value must be String, bool or int")
        }
    }
    /// プロパティの値取得
    pub fn get(&self, key: &str) -> Option<&OwnedValue> {
        self.properties.get(key)
    }
    pub fn inner(&self) -> &HashMap<String, OwnedValue> {
        &self.properties
    }
}

/// MenuPropertiesを作成するマクロ
#[macro_export]
macro_rules! menu_properties {
    // 最後が,で終る場合
    ($($key:expr => $value:expr,)+) => { $crate::menu_properties!{$($key => $value),+}};
    // 間にカンマを使う場合
    ($($key:expr => $value:expr),*) => {
        {
            let mut properties = $crate::dbus::dbusmenu::MenuProperties::new();
            $(
                let _ = properties.insert_value($key.to_owned(), $value);
            )*
            properties
        }
    };
}

/// メニューツリーの要素となるノード
pub struct MenuNode {
    pub id: i32,
    pub properties: MenuProperties,
    pub children: Vec<i32>,
}

#[derive(Default)]
pub struct MenuTree {
    node_map: IndexMap<i32, MenuNode>,
}

impl MenuTree {
    pub fn new() -> Self {
        Self {
            node_map: IndexMap::new(),
        }
    }

    pub fn get_mut(&mut self, id: i32) -> Option<&mut MenuNode> {
        self.node_map.get_mut(&id)
    }

    pub fn insert_node(&mut self, node: MenuNode) {
        self.node_map.insert(node.id, node);
    }

    /// 多くのアプリケーションでは実装されていない．
    pub fn get_property(&self, id: i32, name: &str) -> OwnedValue {
        if let Some(node) = self.node_map.get(&id) {
            if let Some(property_value) = node.properties.get(name) {
                property_value.try_clone().unwrap()
            } else {
                Value::from("").try_into_owned().unwrap()
            }
        } else {
            Value::from("").try_into_owned().unwrap()
        }
    }

    /// 指定したidとプロパティから配列を取得する．
    pub fn get_group_properties(
        &self,
        ids: &[i32],
        properties: &[String],
    ) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        let mut result_list = Vec::<(i32, HashMap<String, OwnedValue>)>::new();

        for id in ids {
            let result_item: (i32, HashMap<String, OwnedValue>) =
                if let Some(node) = self.node_map.get(id) {
                    let partial_properties = if properties.is_empty() {
                        // propertiesに空白リストを渡した場合
                        node.properties.inner().clone()
                    } else {
                        let mut partial_properties = HashMap::<String, OwnedValue>::new();

                        for property in properties {
                            if let Some(property_value) = node.properties.get(property) {
                                partial_properties.insert(property.clone(), property_value.clone());
                            }
                        }

                        partial_properties
                    };

                    (*id, partial_properties)
                } else {
                    (*id, HashMap::<String, OwnedValue>::new())
                };

            result_list.push(result_item);
        }

        result_list
    }

    /// 木構造の`OwnedValue`に変換する．
    pub fn to_tree(
        &self,
        id: i32,
        recursion_depth: i32,
        properties: &[String],
    ) -> (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>) {
        if let Some(value) = self.to_sub_tree(id, 0, recursion_depth, properties) {
            value
        } else {
            (
                id,
                HashMap::<String, OwnedValue>::new(),
                Vec::<OwnedValue>::new(),
            )
        }
    }

    fn to_sub_tree(
        &self,
        id: i32,
        relative_depth: i32,
        limit_relative_depth: i32,
        properties: &[String],
    ) -> Option<(i32, HashMap<String, OwnedValue>, Vec<OwnedValue>)> {
        let node = self.node_map.get(&id)?;

        let mut child_value_list = Vec::<OwnedValue>::new();
        let child_relative_depth = relative_depth + 1;

        if limit_relative_depth == -1 || child_relative_depth <= limit_relative_depth {
            for child_id in &node.children {
                if let Some(child_value) = self.to_sub_tree(
                    *child_id,
                    child_relative_depth,
                    limit_relative_depth,
                    properties,
                ) {
                    child_value_list.push(Value::from(child_value).try_into_owned().unwrap());
                }
            }
        }

        let partial_properties = if properties.is_empty() {
            // propertiesに空白リストを渡した場合
            node.properties.inner().clone()
        } else {
            let mut partial_properties = HashMap::<String, OwnedValue>::new();

            for property in properties {
                if let Some(property_value) = node.properties.get(property) {
                    partial_properties.insert(property.clone(), property_value.clone());
                }
            }

            partial_properties
        };

        Some((id, partial_properties, child_value_list))
    }
}

// pub fn temp_test() {
//     let mut menu_tree = MenuTree::new();

//     let root_node = MenuNode {
//         id: 0,
//         properties: menu_properties!("children-display" => "submenu"),
//         children: vec![100, 101, 2, 305, 3, 4, 5, 6],
//     };
//     menu_tree.insert_node(root_node);

//     menu_tree.insert_node(MenuNode {
//         id: 100,
//         properties: menu_properties!(
//             "label" => "Keyboard - Japanese",
//             "icon-name" => "input-keyboard",
//             "toggle-type" => "radio",
//             "toggle-state" => 0_i32
//         ),
//         children: Vec::new(),
//     });

//     menu_tree.insert_node(MenuNode {
//         id: 101,
//         properties: menu_properties!(
//             "label" => "Mozc",
//             "icon-name" => "fcitx-mozc",
//             "toggle-type" => 1_i32
//         ),
//         children: Vec::new(),
//     });

//     menu_tree.insert_node(MenuNode {
//         id: 2,
//         properties: menu_properties!(
//             "type" => "separator",
//         ),
//         children: Vec::new(),
//     });

//     menu_tree.insert_node(MenuNode {
//         id: 305,
//         properties: menu_properties!(
//             "label" => "Mozc Settings",
//             "icon-name" => "fcitx-mozc-tool",
//             "children-display" => "submenu"
//         ),
//         children: vec![306, 307, 308, 309, 310, 311, 312, 313, 314, 315],
//     });

//     // println!("{:#?}", menu_tree.to_tree(0, 1, &[]));
//     println!("{:#?}", menu_tree.get_group_properties(&[0], &[]))
// }
