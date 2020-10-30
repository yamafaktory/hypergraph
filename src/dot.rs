use crate::core::{HyperedgeVertices, Hypergraph, SharedTrait};
pub(super) use crate::private::ExtendedDebug;

use indexmap::IndexSet;
use random_color::{Luminosity, RandomColor};

/// Render the hypergraph to Graphviz dot format.
pub(super) fn render_to_graphviz_dot<V, HE>(hypergraph: &Hypergraph<V, HE>)
where
    V: SharedTrait,
    HE: SharedTrait,
{
    let indent = |contents: &str| format!("{: >4}{}", String::new(), contents);

    // println!("{:?}", hypergraph.hyperedges);
    // println!("{:?}", hypergraph.vertices);

    // Partition the hyperedges in two groups, one for the unaries, the other for the rest.
    // Graphviz dot doesn't provide a way to render a hypergraph. To overcome this issue,
    // we need to track the unaries and treat them separately.
    let partitioned_hyperedges = hypergraph.hyperedges.iter().fold(
        (Vec::new(), Vec::new()),
        |mut acc: (
            Vec<(&HyperedgeVertices, &IndexSet<HE>)>,
            Vec<(&HyperedgeVertices, &IndexSet<HE>)>,
        ),
         (vertices, weight)| {
            if vertices.len() == 1 {
                acc.1.push((vertices, weight)); // Unaries.
            } else {
                acc.0.push((vertices, weight));
            }

            acc
        },
    );
    // dbg!(partitioned_hyperedges);

    let rendered_vertices =
        hypergraph
            .vertices
            .iter()
            .enumerate()
            .fold(String::new(), |acc, (index, (weight, _))| {
                [
                    indent(format!(r#"{} [label="{:?}"];"#, index, weight.safe_debug()).as_str()),
                    acc,
                ]
                .join("\n")
            });

    // dbg!(t.0);
    // .fold(String::new(), |acc, (vertices, weight)| {
    //     format!(
    //         r#"
    //         {}
    //         {} [ color="{}", label={:?} ];
    //     "#,
    //         acc,
    //         vertices.iter().join(" -> ").as_str(),
    //         RandomColor::new().luminosity(Luminosity::Dark).to_hex(),
    //         weight
    //     )
    // });

    // let test: String =
    //     self.hyperedges
    //         .iter()
    //         .fold(String::new(), |acc, (vertices, weights)| {
    //             let random_color = RandomColor::new().luminosity(Luminosity::Dark).to_hex();

    //             let t = if vertices.len() == 1 {
    //                 dbg!(vertices, weights);
    //                 format!("todo")
    //             } else {
    //                 weights.iter().fold(String::new(), |weights_acc, weight| {
    //                     format!(
    //                         r#"
    //                             {}
    //                             {} [ color="{}", fontcolor="{}", label={:?} ];
    //                         "#,
    //                         weights_acc,
    //                         vertices.iter().join(" -> ").as_str(),
    //                         random_color,
    //                         random_color,
    //                         weight
    //                     )
    //                 })
    //             };

    //             format!(
    //                 r#"
    //                     {}
    //                     {}
    //                 "#,
    //                 acc, t
    //             )
    //         });

    let dot = [
        String::from("digraph {"),
        indent("edge [ penwidth=0.5, arrowhead=normal, arrowsize=0.5, fontsize=8.0 ];"),
        indent("node [ color=gray20, fontsize=8.0, fontcolor=white, style=filled, shape=circle ];"),
        indent("rankdir=LR;"),
        String::from(""),
        rendered_vertices,
        String::from("}"),
    ]
    .join("\n");

    println!("{}", dot);
}
