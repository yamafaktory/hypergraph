use crate::core::debug::ExtendedDebug;
use crate::core::{HyperedgeVertices, Hypergraph, SharedTrait};

use dot_writer::Attributes;
use indexmap::{IndexMap, IndexSet};

#[cfg(test)]
fn get_random_hex_color() -> String {
    String::from("#ffffff")
}

#[cfg(not(test))]
fn get_random_hex_color() -> String {
    use random_color::{Luminosity, RandomColor};

    RandomColor::new().luminosity(Luminosity::Dark).to_hex()
}

pub(super) fn render_to_graphviz_dot<V, HE>(hypergraph: &Hypergraph<V, HE>) -> String
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Partition the hyperedges in two groups, one for the unaries, the other for the rest.
    // Graphviz dot doesn't provide a way to render a hypergraph. To overcome this issue,
    // we need to track the unaries and treat them separately.
    let mut unaries: IndexMap<usize, &IndexSet<HE>> = IndexMap::new();
    let mut non_unaries: Vec<(&HyperedgeVertices, &IndexSet<HE>)> = Vec::new();
    for (vertices, weight) in hypergraph.hyperedges.iter() {
        match vertices.as_slice() {
            [] => {} // TODO nothing to draw for empty sets?
            [vertex] => {
                unaries.insert(*vertex, weight);
            }
            _ => {
                non_unaries.push((vertices, weight));
            }
        }
    }

    let mut output = Vec::new();
    {
        let mut writer = dot_writer::DotWriter::from(&mut output);
        let mut graph = writer.digraph();

        graph
            .edge_attributes()
            .set_pen_width(0.5)
            .set_arrow_head(dot_writer::ArrowType::Normal)
            .set_arrow_size(0.5)
            .set_font_size(8.0);

        graph
            .node_attributes()
            .set_color(dot_writer::Color::Gray20)
            .set_font_size(8.0)
            .set_font_color(dot_writer::Color::White)
            .set_style(dot_writer::Style::Filled)
            .set_shape(dot_writer::Shape::Circle);

        graph.set_rank_direction(dot_writer::RankDirection::LeftRight);

        for (index, (weight, _)) in hypergraph.vertices.iter().enumerate() {
            let mut node = graph.node_named(&index.to_string());
            node.set_label(&format!("{:?}", weight.safe_debug()));
            if let Some(weight) = unaries.get(&index) {
                node.set("peripheries", &(weight.len() + 1).to_string(), false);
            }
        }

        for (vertices, weights) in non_unaries.iter() {
            let random_color = get_random_hex_color();
            for weight in weights.iter() {
                graph
                    .edges(vertices.iter().map(|v| v.to_string()))
                    .unwrap() // shouldn't panic as vertices.len() >= 2 checked above
                    .attributes()
                    .set("color", &random_color, true)
                    .set("fontcolor", &random_color, true)
                    .set_label(&format!("{:?}", weight.safe_debug()));
            }
        }
    }

    String::from_utf8(output).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot() {
        #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
        struct T<'a> {
            name: &'a str,
        }
        let mut graph = Hypergraph::<T<'_>, T<'_>>::new();

        graph.add_vertex(T { name: "a" });
        graph.add_vertex(T { name: "b" });
        graph.add_vertex(T { name: "c" });

        graph.add_hyperedge(&[0, 1, 2], T { name: "foo" });
        graph.add_hyperedge(&[0, 1, 2], T { name: "bar" });
        graph.add_hyperedge(&[0], T { name: "unary" });

        let rendered_graph = render_to_graphviz_dot(&graph);

        assert!(rendered_graph
            .contains(r##"  0 [label="T {\l    name: \"a\",\l}\l", peripheries=2];"##));
        assert!(rendered_graph.contains(r##"  1 [label="T {\l    name: \"b\",\l}\l"];"##));
        assert!(rendered_graph.contains(r##"  2 [label="T {\l    name: \"c\",\l}\l"];"##));

        assert!(rendered_graph.contains(r##"  0 -> 1 -> 2 [color="#ffffff", fontcolor="#ffffff", label="T {\l    name: \"foo\",\l}\l"];"##));
        assert!(rendered_graph.contains(r##"  0 -> 1 -> 2 [color="#ffffff", fontcolor="#ffffff", label="T {\l    name: \"bar\",\l}\l"];"##));
    }
}
