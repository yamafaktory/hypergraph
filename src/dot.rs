use crate::core::{HyperedgeVertices, Hypergraph, SharedTrait};
pub(super) use crate::private::ExtendedDebug;

use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use random_color::{Luminosity, RandomColor};

fn indent(contents: &str) -> String {
    format!("{: >4}{}", String::new(), contents)
}

/// Render the hypergraph to Graphviz dot format.
/// Due to Graphviz dot inability to render hypergraphs out of the box,
/// unaries are rendered as vertex peripheries which can't be labelled.
pub(super) fn render_to_graphviz_dot<V, HE>(hypergraph: &Hypergraph<V, HE>)
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
            Vec<(&HyperedgeVertices, &IndexSet<HE>)>,
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
                        let random_color = RandomColor::new().luminosity(Luminosity::Dark).to_hex();

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
                                Some(weight) => format!(", peripheries={}", weight.len()),
                                None => String::new(),
                            }
                        )
                        .as_str(),
                    ),
                ]
                .join("\n")
            });

    println!(
        "{}",
        [
            String::from("digraph {"),
            indent("edge [penwidth=0.5, arrowhead=normal, arrowsize=0.5, fontsize=8.0];"),
            indent(
                "node [color=gray20, fontsize=8.0, fontcolor=white, style=filled, shape=circle];"
            ),
            indent("rankdir=LR;"),
            vertices,
            non_unary_hyperedges,
            String::from("}"),
        ]
        .join("\n")
    );
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

        graph.add_hyperedge(&[0, 1, 2], T { name: "foo\nbar" });
        graph.add_hyperedge(&[0, 1, 2], T { name: "sdf" });

        graph.render_to_graphviz_dot();
    }
}
