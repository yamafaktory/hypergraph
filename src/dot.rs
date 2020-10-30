use crate::core::{HyperedgeVertices, Hypergraph, SharedTrait};
pub(super) use crate::private::ExtendedDebug;

use indexmap::IndexSet;
use itertools::Itertools;
use random_color::{Luminosity, RandomColor};

fn indent(contents: &str) -> String {
    format!("{: >4}{}", String::new(), contents)
}

/// Render the hypergraph to Graphviz dot format.
pub(super) fn render_to_graphviz_dot<V, HE>(hypergraph: &Hypergraph<V, HE>)
where
    V: SharedTrait,
    HE: SharedTrait,
{
    // Partition the hyperedges in two groups, one for the unaries, the other for the rest.
    // Graphviz dot doesn't provide a way to render a hypergraph. To overcome this issue,
    // we need to track the unaries and treat them separately.
    let partitioned_hyperedges = hypergraph.hyperedges.iter().fold(
        (Vec::new(), Vec::new()),
        |mut acc: (
            Vec<(&HyperedgeVertices, &IndexSet<HE>)>,
            Vec<(usize, &IndexSet<HE>)>,
        ),
         (vertices, weight)| {
            if vertices.len() == 1 {
                acc.1.push((vertices[0], weight)); // Unaries.
            } else {
                acc.0.push((vertices, weight));
            }

            acc
        },
    );

    let find_unary = |index: usize| {
        partitioned_hyperedges
            .1
            .iter()
            .find_position(|(idx, _)| *idx == index)
    };

    let non_unary_hyperedges =
        partitioned_hyperedges
            .0
            .into_iter()
            .fold(String::new(), |acc, (vertices, weight)| {
                let random_color = RandomColor::new().luminosity(Luminosity::Dark).to_hex();

                [
                    acc,
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
            });

    let rendered_vertices =
        hypergraph
            .vertices
            .iter()
            .enumerate()
            .fold(String::new(), |acc, (index, (weight, _))| {
                [
                    acc,
                    indent(format!(r#"{} [label="{:?}"];"#, index, weight.safe_debug()).as_str()),
                ]
                .join("\n")
            });

    dbg!(partitioned_hyperedges.1);
    
    // let t = partitioned_hyperedges
    //     .1
    //     .into_iter()
    //     .fold(String::new(), |acc, (vertices, weight)| {
    //         [
    //             acc,
    //             indent(
    //                 format!(
    //                     r#"{} [label="{:?}", peripheries={}];"#,
    //                     vertices[0],
    //                     "test",
    //                     weight.len()
    //                 )
    //                 .as_str(),
    //             ),
    //         ]
    //         .join("\n")
    //     });

    let dot = [
        String::from("digraph {"),
        indent("edge [penwidth=0.5, arrowhead=normal, arrowsize=0.5, fontsize=8.0];"),
        indent("node [color=gray20, fontsize=8.0, fontcolor=white, style=filled, shape=circle];"),
        indent("rankdir=LR;"),
        String::from(""),
        rendered_vertices,
        non_unary_hyperedges,
        String::from("}"),
    ]
    .join("\n");

    println!("{}", dot);
}
