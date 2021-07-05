use crate::core::debug::ExtendedDebug;
use crate::core::{Hypergraph, SharedTrait};
use crate::UnstableVertexIndex;

use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;

fn indent(contents: &str) -> String {
    format!("{: >4}{}", String::new(), contents)
}

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
    let partitioned_hyperedges = hypergraph.hyperedges.iter().fold(
        (Vec::new(), IndexMap::new()),
        #[allow(clippy::type_complexity)]
        |mut acc: (
            Vec<(&Vec<UnstableVertexIndex>, &IndexSet<HE>)>,
            IndexMap<usize, &IndexSet<HE>>,
        ),
         (vertices, weight)| {
            if vertices.len() == 1 {
                acc.1.insert(vertices[0], weight); // Unaries.
            } else {
                acc.0.push((vertices, weight));
            }

            acc
        },
    );

    let non_unary_hyperedges =
        partitioned_hyperedges
            .0
            .iter()
            .fold(String::new(), |acc, (vertices, weight)| {
                [
                    acc,
                    weight.iter().fold(String::new(), |weight_acc, weight| {
                        let random_color = get_random_hex_color();

                        [
                            weight_acc,
                            indent(
                                format!(
                                    r#"{} [color="{}", fontcolor="{}", label="{:?}"];"#,
                                    vertices.iter().join(" -> ").as_str(),
                                    random_color,
                                    random_color,
                                    weight.safe_debug()
                                )
                                .as_str(),
                            ),
                        ]
                        .join("\n")
                    }),
                ]
                .join("\n")
            });

    let vertices =
        hypergraph
            .vertices
            .iter()
            .enumerate()
            .fold(String::new(), |acc, (index, (weight, _))| {
                [
                    acc,
                    indent(
                        format!(
                            r#"{} [label="{:?}"{}];"#,
                            index,
                            weight.safe_debug(),
                            // Inject peripheries for unaries.
                            match partitioned_hyperedges.1.get(&index) {
                                Some(weight) => format!(", peripheries={}", weight.len() + 1),
                                None => String::new(),
                            }
                        )
                        .as_str(),
                    ),
                ]
                .join("\n")
            });

    // Return the rendered graph as String.
    [
        String::from("digraph {"),
        indent("edge [penwidth=0.5, arrowhead=normal, arrowsize=0.5, fontsize=8.0];"),
        indent("node [color=gray20, fontsize=8.0, fontcolor=white, style=filled, shape=circle];"),
        indent("rankdir=LR;"),
        vertices,
        non_unary_hyperedges,
        String::from("}"),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::StableVertexIndex;

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

        graph.add_hyperedge(
            vec![
                StableVertexIndex(0),
                StableVertexIndex(1),
                StableVertexIndex(2),
            ],
            T { name: "foo" },
        );
        graph.add_hyperedge(
            vec![
                StableVertexIndex(0),
                StableVertexIndex(1),
                StableVertexIndex(2),
            ],
            T { name: "bar" },
        );
        graph.add_hyperedge(vec![StableVertexIndex(0)], T { name: "unary" });

        let rendered_graph = render_to_graphviz_dot(&graph);

        assert!(
            rendered_graph.contains(r##"0 [label="T {\l    name: \"a\",\l}\l", peripheries=2];"##)
        );
        assert!(rendered_graph.contains(r##"1 [label="T {\l    name: \"b\",\l}\l"];"##));
        assert!(rendered_graph.contains(r##"2 [label="T {\l    name: \"c\",\l}\l"];"##));

        assert!(rendered_graph.contains(r##"0 -> 1 -> 2 [color="#ffffff", fontcolor="#ffffff", label="T {\l    name: \"foo\",\l}\l"];"##));
        assert!(rendered_graph.contains(r##"0 -> 1 -> 2 [color="#ffffff", fontcolor="#ffffff", label="T {\l    name: \"bar\",\l}\l"];"##));
    }
}
