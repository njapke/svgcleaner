/****************************************************************************
**
** svgcleaner could help you to clean up your SVG files
** from unnecessary data.
** Copyright (C) 2012-2016 Evgeniy Reizner
**
** This program is free software; you can redistribute it and/or modify
** it under the terms of the GNU General Public License as published by
** the Free Software Foundation; either version 2 of the License, or
** (at your option) any later version.
**
** This program is distributed in the hope that it will be useful,
** but WITHOUT ANY WARRANTY; without even the implied warranty of
** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
** GNU General Public License for more details.
**
** You should have received a copy of the GNU General Public License along
** with this program; if not, write to the Free Software Foundation, Inc.,
** 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
**
****************************************************************************/

use super::short::{EId, AId};

use svgdom::{Document, Node, AttributeValue, ValueId};

// TODO: process mask element
// TODO: process visibility

pub fn remove_invisible_elements(doc: &Document) {
    let mut is_any_removed = false;
    process_display_attribute(doc, &mut is_any_removed);
    process_paths(doc, &mut is_any_removed);
    process_clip_paths(doc, &mut is_any_removed);
    process_filter(doc, &mut is_any_removed);

    if is_any_removed {
        super::remove_unused_defs(&doc);
    }
}

// Remove invalid elements from 'clipPath' and if 'clipPath' is empty or became empty
// - remove it and all elements that became invalid or unused.
fn process_clip_paths(doc: &Document, is_any_removed: &mut bool) {
    let mut nodes = Vec::with_capacity(16);
    let mut clip_paths = Vec::with_capacity(16);

    // remove all invalid children
    for node in doc.descendants().filter(|n| n.is_tag_id(EId::ClipPath)) {
        for child in node.children() {
            if !is_valid_clip_path_elem(&child) {
                nodes.push(child.clone());
            }
        }

        while let Some(n) = nodes.pop() {
            n.remove();
        }

        if !node.has_children() {
            clip_paths.push(node.clone());
        }
    }

    if !clip_paths.is_empty() {
        *is_any_removed = true;
    }

    // Remove empty clipPath's.
    // Note, that all elements that uses this clip path also became invisible,
    // so we can remove them as well.
    while let Some(n) = clip_paths.pop() {
        for link in n.linked_nodes() {
            link.remove();
        }
        n.remove();
    }
}

fn is_valid_clip_path_elem(node: &Node) -> bool {
    // https://www.w3.org/TR/SVG/masking.html#EstablishingANewClippingPath

    fn is_valid_shape(node: &Node) -> bool {
           node.is_basic_shape()
        || node.is_tag_id(EId::Path)
        || node.is_tag_id(EId::Text)
    }

    if node.is_tag_id(EId::Use) {
        if !node.has_attribute(AId::XlinkHref) {
            return false;
        }

        match node.attribute_value(AId::XlinkHref).unwrap() {
            AttributeValue::Link(link) => {
                return is_valid_shape(&link);
            }
            _ => {}
        }
    }

    is_valid_shape(node)
}

// Paths with empty 'd' attribute are invisible and we can remove them.
fn process_paths(doc: &Document, is_any_removed: &mut bool) {
    let mut paths = Vec::with_capacity(16);

    fn is_invisible(node: &Node) -> bool {
        if node.has_attribute(AId::D) {
            let attrs = node.attributes();
            match attrs.get_value(AId::D).unwrap() {
                &AttributeValue::Path(ref d) => {
                    if d.d.is_empty() {
                        return true;
                    }
                }
                // invalid value type
                _ => return true,
            }
        } else {
            // not set
            return true;
        }

        return false;
    }

    for node in doc.descendants().filter(|n| n.is_tag_id(EId::Path)) {
        if is_invisible(&node) {
            paths.push(node.clone());
        }
    }

    if !paths.is_empty() {
        *is_any_removed = true;
    }

    while let Some(n) = paths.pop() {
        n.remove();
    }
}

// Remove elements with 'display:none'.
fn process_display_attribute(doc: &Document, is_any_removed: &mut bool) {
    let mut nodes = Vec::with_capacity(16);

    let mut iter = doc.descendants();
    while let Some(node) = iter.next() {
        // if elements has attribute 'display:none' and this element is not used - we can remove it
        if node.has_attribute_with_value(AId::Display, ValueId::None) && !node.is_used() {
            // all children must be unused to
            if !node.descendants().any(|n| n.is_used()) {
                // TODO: ungroup used elements and remove unused
                nodes.push(node.clone());

                if node.has_children() {
                    iter.skip_children();
                }
            }
        }
    }

    if !nodes.is_empty() {
        *is_any_removed = true;
    }

    while let Some(n) = nodes.pop() {
        n.remove();
    }
}

// remove 'filter' elements without children
fn process_filter(doc: &Document, is_any_removed: &mut bool) {
    let nodes: Vec<Node> = doc.descendants()
                              .filter(|n| n.is_tag_id(EId::Filter) && !n.has_children()).collect();

    if !nodes.is_empty() {
        *is_any_removed = true;
    }

    // Note, that all elements that uses this filter also became invisible,
    // so we can remove them as well.
    for n in nodes {
        for link in n.linked_nodes() {
            link.remove();
        }
        n.remove();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use svgdom::{Document, WriteToString};
    use task::{group_defs, final_fixes};

    macro_rules! test {
        ($name:ident, $in_text:expr, $out_text:expr) => (
            #[test]
            fn $name() {
                let doc = Document::from_data($in_text).unwrap();
                // we must prepare defs, because `remove_invisible_elements`
                // invokes `remove_unused_defs`
                group_defs(&doc);
                remove_invisible_elements(&doc);
                // removes `defs` element
                final_fixes(&doc);
                assert_eq_text!(doc.to_string_with_opt(&write_opt_for_tests!()), $out_text);
            }
        )
    }

    macro_rules! test_eq {
        ($name:ident, $in_text:expr) => (
            test!($name, $in_text, String::from_utf8_lossy($in_text));
        )
    }

    test!(rm_clip_path_children_1,
b"<svg>
    <clipPath>
        <g/>
        <rect/>
    </clipPath>
</svg>",
"<svg>
    <clipPath>
        <rect/>
    </clipPath>
</svg>
");

    test!(rm_clip_path_children_2,
b"<svg>
    <clipPath>
        <use/>
        <use xlink:href='#g1'/>
        <use xlink:href='#rect1'/>
    </clipPath>
    <rect id='rect1'/>
    <g id='g1'/>
</svg>",
"<svg>
    <clipPath>
        <use xlink:href='#rect1'/>
    </clipPath>
    <rect id='rect1'/>
    <g id='g1'/>
</svg>
");

    test!(rm_clip_path_1,
b"<svg>
    <clipPath id='cp1'/>
    <rect clip-path='url(#cp1)'/>
    <rect clip-path='url(#cp1)'/>
</svg>",
"<svg/>
");

    test!(rm_clip_path_2,
b"<svg>
    <linearGradient id='lg1'/>
    <clipPath id='cp1'/>
    <rect clip-path='url(#cp1)' fill='url(#lg1)'/>
</svg>",
"<svg/>
");

    test!(rm_clip_path_3,
b"<svg>
    <clipPath>
        <rect display='none'/>
    </clipPath>
</svg>",
"<svg/>
");

    test!(rm_path_1,
b"<svg>
    <path/>
</svg>",
"<svg/>
");

    test!(rm_path_2,
b"<svg>
    <path d=''/>
</svg>",
"<svg/>
");

    test!(rm_path_3,
b"<svg>
    <linearGradient id='lg1'/>
    <path d='' fill='url(#lg1)'/>
</svg>",
"<svg/>
");

    test!(rm_display_none_1,
b"<svg>
    <path display='none'/>
</svg>",
"<svg/>
");

    test!(rm_display_none_2,
b"<svg>
    <g display='none'>
        <rect/>
    </g>
</svg>",
"<svg/>
");

    test_eq!(skip_display_none_1,
b"<svg>
    <g display='none'>
        <rect id='r1'/>
    </g>
    <use xlink:href='#r1'/>
</svg>
");

    test!(rm_filter_1,
b"<svg>
    <filter/>
</svg>",
"<svg/>
");

    test!(rm_filter_2,
b"<svg>
    <filter id='f1'/>
    <rect filter='url(#f1)'/>
</svg>",
"<svg/>
");

}