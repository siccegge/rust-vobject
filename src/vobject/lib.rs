#![feature(phase)]
#[phase(plugin)]
extern crate peg_syntax_ext;

use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};

pub struct Property {
    raw_params: String,
    raw_value: String,
}

impl Property {
    fn new(raw_params: String, raw_value: String) -> Property {
        Property {
            raw_params: raw_params,
            raw_value: raw_value
        }
    }

    #[doc="Get parameters as unparsed string."]
    pub fn get_raw_params(&self) -> &String { &self.raw_params }

    #[doc="Get value as unparsed string."]
    pub fn get_raw_value(&self) -> &String { &self.raw_value }
}


pub struct Component {
    #[doc="The name of the component, such as `VCARD` or `VEVENT`."]
    pub name: String,

    #[doc="The component's properties."]
    pub props: HashMap<String, Vec<Property>>
}

impl Component {
    fn new(name: String) -> Component {
        Component {
            name: name,
            props: HashMap::new()
        }
    }

    #[doc="Retrieve one property (from many) by key.
        Returns `None` if nothing is found."]
    pub fn single_prop(&self, key: &String) -> Option<&Property> {
        match self.props.get(key) {
            Some(x) => {
                match x.len() {
                    1 => Some(&x[0]),
                    _ => None
                }
            },
            None => None
        }
    }

    #[doc="Retrieve a mutable vector of properties for this key.
        Creates one (and inserts it into the component) if none exists."]
    pub fn all_props_mut(&mut self, key: String) -> &mut Vec<Property> {
        match self.props.entry(key) {
            Occupied(values) => values.into_mut(),
            Vacant(values) => values.set(vec![])
        }
    }

    #[doc="Retrieve properties by key.
        Returns an empty slice if key doesn't exist."]
    pub fn all_props(&self, key: &String) -> &[Property] {
        static EMPTY: &'static [Property] = [];
        match self.props.get(key) {
            Some(values) => values.as_slice(),
            None => EMPTY
        }
    }
}


peg! parser(r#"
use super::{Component,Property};

#[pub]
component -> Component
    = name:component_begin (eol+) ps:props (eol+) component_end __ {
        let mut rv = Component::new(name);

        for (k, v) in ps.into_iter() {
            rv.all_props_mut(k).push(v);
        };
        rv
    }

component_begin -> String
    = "BEGIN:" v:prop_value { v }

component_end -> String
    = "END:" v:prop_value { v }

props -> Vec<(String, Property)>
    = ps:prop ++ (eol+) { ps }

prop -> (String, Property)
    = k:prop_name p:(";" p:prop_params {p})? ":" v:prop_value {
        let params = match p {
            Some(x) => x,
            None => "".into_string()
        };
        (k, Property::new(params, v))
    }

prop_name -> String
    = !"BEGIN" !"END" name_char+ { match_str.into_string() }

prop_params -> String
    = prop_char+ { match_str.into_string() }

prop_value -> String
    = value_char+ { match_str.into_string() }

// Characters
name_char = ([a-zA-Z] / "-")
prop_char = name_char / [=;]
value_char = !eol .

eol = "\n" / "\r\n" / "\r"
whitespace = " " / "\t"
__ = (eol / whitespace)*

"#)


#[doc="Parse a component. The error value is a human-readable message."]
pub fn parse_component(s: &String) -> Result<Component, String> {
    // XXX: The unfolding should be worked into the PEG
    // See feature request: https://github.com/kevinmehall/rust-peg/issues/26
    let unfolded = s
        .replace("\n ", "").replace("\n\t", "")
        .replace("\r\n ", "").replace("\r\n\t", "")
        .replace("\r ", "").replace("\r\t", "");

    parser::component(unfolded.as_slice())
}
