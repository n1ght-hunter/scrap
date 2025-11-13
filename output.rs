Crate {
    id: NodeId(4294967040),
    attrs: [],
    items: [
        Item {
            attrs: [],
            id: NodeId(4294967040),
            span: test.rs:1:1: 1:13 (#0),
            vis: Visibility {
                kind: Inherited,
                span: no-location (#0),
                tokens: None,
            },
            kind: Use(
                UseTree {
                    prefix: Path {
                        span: test.rs:1:5: 1:12 (#0),
                        segments: [
                            PathSegment {
                                ident: std#0,
                                id: NodeId(4294967040),
                                args: None,
                            },
                            PathSegment {
                                ident: io#0,
                                id: NodeId(4294967040),
                                args: None,
                            },
                        ],
                        tokens: None,
                    },
                    kind: Simple(
                        None,
                    ),
                    span: test.rs:1:5: 1:12 (#0),
                },
            ),
            tokens: None,
        },
        Item {
            attrs: [],
            id: NodeId(4294967040),
            span: test.rs:3:1: 6:2 (#0),
            vis: Visibility {
                kind: Inherited,
                span: test.rs:3:1: 3:1 (#0),
                tokens: None,
            },
            kind: Fn(
                Fn {
                    defaultness: Final,
                    ident: main#0,
                    generics: Generics {
                        params: [],
                        where_clause: WhereClause {
                            has_where_token: false,
                            predicates: [],
                            span: test.rs:3:10: 3:10 (#0),
                        },
                        span: test.rs:3:8: 3:8 (#0),
                    },
                    sig: FnSig {
                        header: FnHeader {
                            constness: No,
                            coroutine_kind: None,
                            safety: Default,
                            ext: None,
                        },
                        decl: FnDecl {
                            inputs: [],
                            output: Default(
                                test.rs:3:10: 3:10 (#0),
                            ),
                        },
                        span: test.rs:3:1: 3:10 (#0),
                    },
                    contract: None,
                    define_opaque: None,
                    body: Some(
                        Block {
                            stmts: [
                                Stmt {
                                    id: NodeId(4294967040),
                                    kind: Item(
                                        Item {
                                            attrs: [],
                                            id: NodeId(4294967040),
                                            span: test.rs:4:5: 4:20 (#0),
                                            vis: Visibility {
                                                kind: Inherited,
                                                span: test.rs:4:5: 4:5 (#0),
                                                tokens: None,
                                            },
                                            kind: Use(
                                                UseTree {
                                                    prefix: Path {
                                                        span: test.rs:4:9: 4:19 (#0),
                                                        segments: [
                                                            PathSegment {
                                                                ident: core#0,
                                                                id: NodeId(4294967040),
                                                                args: None,
                                                            },
                                                            PathSegment {
                                                                ident: asdf#0,
                                                                id: NodeId(4294967040),
                                                                args: None,
                                                            },
                                                        ],
                                                        tokens: None,
                                                    },
                                                    kind: Simple(
                                                        None,
                                                    ),
                                                    span: test.rs:4:9: 4:19 (#0),
                                                },
                                            ),
                                            tokens: None,
                                        },
                                    ),
                                    span: test.rs:4:5: 4:20 (#0),
                                },
                                Stmt {
                                    id: NodeId(4294967040),
                                    kind: Semi(
                                        Expr {
                                            id: NodeId(4294967040),
                                            kind: Call(
                                                Expr {
                                                    id: NodeId(4294967040),
                                                    kind: Path(
                                                        None,
                                                        Path {
                                                            span: test.rs:5:5: 5:9 (#0),
                                                            segments: [
                                                                PathSegment {
                                                                    ident: test#0,
                                                                    id: NodeId(4294967040),
                                                                    args: None,
                                                                },
                                                            ],
                                                            tokens: None,
                                                        },
                                                    ),
                                                    span: test.rs:5:5: 5:9 (#0),
                                                    attrs: [],
                                                    tokens: None,
                                                },
                                                [
                                                    Expr {
                                                        id: NodeId(4294967040),
                                                        kind: Lit(
                                                            Lit {
                                                                kind: Str,
                                                                symbol: "hello world",
                                                                suffix: None,
                                                            },
                                                        ),
                                                        span: test.rs:5:10: 5:23 (#0),
                                                        attrs: [],
                                                        tokens: None,
                                                    },
                                                ],
                                            ),
                                            span: test.rs:5:5: 5:24 (#0),
                                            attrs: [],
                                            tokens: None,
                                        },
                                    ),
                                    span: test.rs:5:5: 5:25 (#0),
                                },
                            ],
                            id: NodeId(4294967040),
                            rules: Default,
                            span: test.rs:3:11: 6:2 (#0),
                            tokens: None,
                        },
                    ),
                },
            ),
            tokens: None,
        },
    ],
    spans: ModSpans {
        inner_span: test.rs:1:1: 6:2 (#0),
        inject_use_span: no-location (#0),
    },
    is_placeholder: false,
}