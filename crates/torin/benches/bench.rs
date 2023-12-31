use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::fmt::Display;
use torin::prelude::*;

struct TestingMeasurer;

impl LayoutMeasurer<usize> for TestingMeasurer {
    fn measure(
        &mut self,
        _node_id: usize,
        _node: &Node,
        _area: &Area,
        _parent_size: &Area,
        _available_parent_area: &Area,
    ) -> Option<Area> {
        None
    }
}

#[derive(Default)]
struct TestingDOM {
    mapper: HashMap<usize, (Option<usize>, Vec<usize>, u16, Node)>,
}

impl TestingDOM {
    fn add(&mut self, node_id: usize, parent: Option<usize>, children: Vec<usize>, node: Node) {
        let depth = parent.map(|p| self.mapper.get(&p).unwrap().2).unwrap_or(0) + 1;
        self.mapper.insert(node_id, (parent, children, depth, node));
    }

    fn add_with_depth(
        &mut self,
        node_id: usize,
        parent: Option<usize>,
        children: Vec<usize>,
        node: Node,
        depth: u16,
    ) {
        self.mapper.insert(node_id, (parent, children, depth, node));
    }

    fn set_node(&mut self, node_id: usize, node: Node) {
        self.mapper.get_mut(&node_id).unwrap().3 = node;
    }
}

impl DOMAdapter<usize> for TestingDOM {
    fn children_of(&mut self, node_id: &usize) -> Vec<usize> {
        self.mapper
            .get(node_id)
            .map(|c| c.1.clone())
            .unwrap_or_default()
    }

    fn parent_of(&self, node_id: &usize) -> Option<usize> {
        self.mapper.get(node_id).map(|c| c.0).flatten()
    }

    fn height(&self, node_id: &usize) -> Option<u16> {
        self.mapper.get(node_id).map(|c| c.2)
    }

    fn get_node(&self, node_id: &usize) -> Option<Node> {
        self.mapper.get(node_id).map(|c| c.3.clone())
    }

    fn is_node_valid(&mut self, _node_id: &usize) -> bool {
        true
    }

    fn closest_common_parent(&self, node_id_a: &usize, _node_id_b: &usize) -> Option<usize> {
        self.parent_of(node_id_a)
    }
}

struct BenchmarkConfig {
    depth: usize,
    wide: usize,
    mode: BenchmarkMode,
    sample: usize,
}

impl BenchmarkConfig {
    pub fn name(&self) -> String {
        format!(
            "size={} depth={} wide={} mode={}",
            self.size(),
            self.depth,
            self.wide,
            self.mode
        )
    }

    pub fn size(&self) -> usize {
        let mut acc = 1; // Root
        let mut prev = 1;

        for _ in 0..self.depth - 1 {
            prev *= self.wide;
            acc += prev;
        }

        acc
    }
}

#[derive(PartialEq, Eq)]
enum BenchmarkMode {
    NoCache,
    InvalidatedCache,
}

impl Display for BenchmarkMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCache => f.write_str("not cached"),
            Self::InvalidatedCache => f.write_str("cached"),
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("benchmarks");

    let benchmarks = [
        BenchmarkConfig {
            depth: 2,
            wide: 1000,
            mode: BenchmarkMode::NoCache,
            sample: 500,
        },
        BenchmarkConfig {
            depth: 2,
            wide: 10000,
            mode: BenchmarkMode::NoCache,
            sample: 500,
        },
        BenchmarkConfig {
            depth: 2,
            wide: 100000,
            mode: BenchmarkMode::NoCache,
            sample: 500,
        },
        BenchmarkConfig {
            depth: 12,
            wide: 2,
            mode: BenchmarkMode::NoCache,
            sample: 500,
        },
        BenchmarkConfig {
            depth: 14,
            wide: 2,
            mode: BenchmarkMode::NoCache,
            sample: 50,
        },
        BenchmarkConfig {
            depth: 17,
            wide: 2,
            mode: BenchmarkMode::NoCache,
            sample: 30,
        },
        BenchmarkConfig {
            depth: 5,
            wide: 15,
            mode: BenchmarkMode::NoCache,
            sample: 500,
        },
        BenchmarkConfig {
            depth: 5,
            wide: 15,
            mode: BenchmarkMode::InvalidatedCache,
            sample: 500,
        },
        BenchmarkConfig {
            depth: 7,
            wide: 5,
            mode: BenchmarkMode::NoCache,
            sample: 50,
        },
        BenchmarkConfig {
            depth: 7,
            wide: 5,
            mode: BenchmarkMode::InvalidatedCache,
            sample: 50,
        },
    ];

    for bench in benchmarks {
        let name = bench.name();
        let BenchmarkConfig {
            depth,
            mode,
            wide,
            sample,
        } = bench;

        g.significance_level(0.05).sample_size(sample);

        g.bench_function(name, |b| {
            let root_area = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1000.0, 1000.0));
            b.iter_batched(
                || {
                    let mut measurer = Some(TestingMeasurer);
                    let mut mocked_dom = TestingDOM::default();

                    mocked_dom.add(
                        0,
                        None,
                        (0..wide - 1).map(|i| (i + 1) + 100).collect(),
                        Node::from_size_and_direction(
                            Size::Percentage(Length::new(100.0)),
                            Size::Percentage(Length::new(100.0)),
                            DirectionMode::Vertical,
                        ),
                    );

                    fn build_branch(
                        mocked_dom: &mut TestingDOM,
                        root: usize,
                        level: usize,

                        depth: usize,
                        wide: usize,

                        mid_node: &mut usize,
                    ) -> Vec<usize> {
                        if level == depth - 1 {
                            return vec![];
                        }

                        let nodes = (0..=wide - 1)
                            .map(|i| i + ((level + 1) * 100) + (root * 10))
                            .into_iter()
                            .collect::<Vec<usize>>();
                        for (i, id) in nodes.iter().enumerate() {
                            if level == depth / 2 && i == nodes.len() / 2 {
                                *mid_node = *id;
                            }

                            let children =
                                build_branch(mocked_dom, *id, level + 1, depth, wide, mid_node);
                            mocked_dom.add_with_depth(
                                *id,
                                Some(root),
                                children,
                                Node::from_size_and_direction(
                                    Size::Pixels(Length::new(100.0)),
                                    Size::Pixels(Length::new(100.0)),
                                    DirectionMode::Vertical,
                                ),
                                level as u16,
                            );
                        }
                        nodes
                    }

                    let mut invalidate_node = 0;
                    build_branch(&mut mocked_dom, 0, 0, depth, wide, &mut invalidate_node);

                    let mut layout = Torin::<usize>::new();

                    if mode == BenchmarkMode::InvalidatedCache {
                        layout.find_best_root(&mut mocked_dom);
                        layout.measure(0, root_area, &mut measurer, &mut mocked_dom);
                        mocked_dom.set_node(
                            invalidate_node,
                            Node::from_size_and_direction(
                                Size::Inner,
                                Size::Pixels(Length::new(10.0)),
                                DirectionMode::Vertical,
                            ),
                        );
                        layout.invalidate(invalidate_node);
                    }

                    (mocked_dom, measurer, layout)
                },
                |(mut mocked_dom, mut measurer, mut layout)| {
                    layout.find_best_root(&mut mocked_dom);
                    layout.measure(0, root_area, &mut measurer, &mut mocked_dom)
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
