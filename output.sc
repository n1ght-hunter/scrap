[2m2025-11-17T06:19:20.145991Z[0m [32m INFO[0m [2mscrap_driver[0m[2m:[0m Loading database snapshot from './target/scrap/complex/cache.json'
[2m2025-11-17T06:19:20.151987Z[0m [32m INFO[0m [2msalsa::function::execute[0m[2m:[0m new_span(Id(4800)): executing query
Can {
    id: NodeId {
        id: 0,
        file_hash: 15139399790505425484,
    },
    items: [
        Item {
            kind: Module(
                Path {
                    span: Span {
                        [salsa id]: Id(1834),
                        start: 0,
                        end: 19,
                    },
                    segments: [
                        PathSegment {
                            ident: Ident {
                                id: NodeId {
                                    id: -2,
                                    file_hash: 0,
                                },
                                name: Symbol {
                                    text: "complex",
                                },
                                span: Span {
                                    [salsa id]: Id(1832),
                                    start: 0,
                                    end: 7,
                                },
                            },
                            id: NodeId {
                                id: -2,
                                file_hash: 0,
                            },
                        },
                        PathSegment {
                            ident: Ident {
                                id: NodeId {
                                    id: 1,
                                    file_hash: 15139399790505425484,
                                },
                                name: Symbol {
                                    text: "external_module",
                                },
                                span: Span {
                                    [salsa id]: Id(1801),
                                    start: 4,
                                    end: 19,
                                },
                            },
                            id: NodeId {
                                id: 1,
                                file_hash: 15139399790505425484,
                            },
                        },
                    ],
                },
                Loaded(
                    [
                        Item {
                            kind: Fn(
                                FnDef {
                                    [salsa id]: Id(3800),
                                    id: NodeId {
                                        id: 9,
                                        file_hash: 18305248712806228625,
                                    },
                                    ident: Ident {
                                        id: NodeId {
                                            id: 0,
                                            file_hash: 18305248712806228625,
                                        },
                                        name: Symbol {
                                            text: "greet",
                                        },
                                        span: Span {
                                            [salsa id]: Id(1c02),
                                            start: 7,
                                            end: 12,
                                        },
                                    },
                                    args: [],
                                    ret_type: None,
                                    body: Block {
                                        stmts: [
                                            Stmt {
                                                id: NodeId {
                                                    id: 7,
                                                    file_hash: 18305248712806228625,
                                                },
                                                kind: Semi(
                                                    Expr {
                                                        id: NodeId {
                                                            id: 6,
                                                            file_hash: 18305248712806228625,
                                                        },
                                                        kind: Call(
                                                            Expr {
                                                                id: NodeId {
                                                                    id: 5,
                                                                    file_hash: 18305248712806228625,
                                                                },
                                                                kind: Path(
                                                                    Path {
                                                                        span: Span {
                                                                            [salsa id]: Id(1c0f),
                                                                            start: 22,
                                                                            end: 27,
                                                                        },
                                                                        segments: [
                                                                            PathSegment {
                                                                                ident: Ident {
                                                                                    id: NodeId {
                                                                                        id: 1,
                                                                                        file_hash: 18305248712806228625,
                                                                                    },
                                                                                    name: Symbol {
                                                                                        text: "print",
                                                                                    },
                                                                                    span: Span {
                                                                                        [salsa id]: Id(1c06),
                                                                                        start: 22,
                                                                                        end: 27,
                                                                                    },
                                                                                },
                                                                                id: NodeId {
                                                                                    id: 2,
                                                                                    file_hash: 18305248712806228625,
                                                                                },
                                                                            },
                                                                        ],
                                                                    },
                                                                ),
                                                                span: Span {
                                                                    [salsa id]: Id(1c0f),
                                                                    start: 22,
                                                                    end: 27,
                                                                },
                                                            },
                                                            [
                                                                Expr {
                                                                    id: NodeId {
                                                                        id: 4,
                                                                        file_hash: 18305248712806228625,
                                                                    },
                                                                    kind: Lit(
                                                                        Lit {
                                                                            id: NodeId {
                                                                                id: 3,
                                                                                file_hash: 18305248712806228625,
                                                                            },
                                                                            kind: Str,
                                                                            span: Span {
                                                                                [salsa id]: Id(1c08),
                                                                                start: 28,
                                                                                end: 57,
                                                                            },
                                                                        },
                                                                    ),
                                                                    span: Span {
                                                                        [salsa id]: Id(1c10),
                                                                        start: 28,
                                                                        end: 58,
                                                                    },
                                                                },
                                                            ],
                                                        ),
                                                        span: Span {
                                                            [salsa id]: Id(1c11),
                                                            start: 22,
                                                            end: 59,
                                                        },
                                                    },
                                                ),
                                                span: Span {
                                                    [salsa id]: Id(1c11),
                                                    start: 22,
                                                    end: 59,
                                                },
                                            },
                                        ],
                                        id: NodeId {
                                            id: 8,
                                            file_hash: 18305248712806228625,
                                        },
                                        span: Span {
                                            [salsa id]: Id(1c12),
                                            start: 15,
                                            end: 62,
                                        },
                                    },
                                    span: Span {
                                        [salsa id]: Id(1c13),
                                        start: 4,
                                        end: 62,
                                    },
                                },
                            ),
                            span: Span {
                                [salsa id]: Id(1c14),
                                start: 0,
                                end: 62,
                            },
                            id: NodeId {
                                id: 10,
                                file_hash: 18305248712806228625,
                            },
                            vis: Visibility {
                                kind: Public,
                                span: Span {
                                    [salsa id]: Id(1c01),
                                    start: 4,
                                    end: 6,
                                },
                            },
                        },
                    ],
                    No,
                    Span {
                        [salsa id]: Id(4c00),
                        start: 0,
                        end: 0,
                    },
                ),
            ),
            span: Span {
                [salsa id]: Id(1835),
                start: 0,
                end: 25,
            },
            id: NodeId {
                id: 2,
                file_hash: 15139399790505425484,
            },
            vis: Visibility {
                kind: Inherited,
                span: Span {
                    [salsa id]: Id(1800),
                    start: 0,
                    end: 3,
                },
            },
        },
        Item {
            kind: Use(
                UseTree {
                    prefix: Path {
                        span: Span {
                            [salsa id]: Id(1836),
                            start: 22,
                            end: 51,
                        },
                        segments: [
                            PathSegment {
                                ident: Ident {
                                    id: NodeId {
                                        id: 3,
                                        file_hash: 15139399790505425484,
                                    },
                                    name: Symbol {
                                        text: "inline_module",
                                    },
                                    span: Span {
                                        [salsa id]: Id(1804),
                                        start: 26,
                                        end: 39,
                                    },
                                },
                                id: NodeId {
                                    id: 4,
                                    file_hash: 15139399790505425484,
                                },
                            },
                            PathSegment {
                                ident: Ident {
                                    id: NodeId {
                                        id: 5,
                                        file_hash: 15139399790505425484,
                                    },
                                    name: Symbol {
                                        text: "greet",
                                    },
                                    span: Span {
                                        [salsa id]: Id(1806),
                                        start: 41,
                                        end: 46,
                                    },
                                },
                                id: NodeId {
                                    id: 6,
                                    file_hash: 15139399790505425484,
                                },
                            },
                        ],
                    },
                    kind: Simple(
                        None,
                    ),
                    span: Span {
                        [salsa id]: Id(1837),
                        start: 22,
                        end: 51,
                    },
                },
            ),
            span: Span {
                [salsa id]: Id(1838),
                start: 22,
                end: 51,
            },
            id: NodeId {
                id: 7,
                file_hash: 15139399790505425484,
            },
            vis: Visibility {
                kind: Inherited,
                span: Span {
                    [salsa id]: Id(1803),
                    start: 22,
                    end: 25,
                },
            },
        },
        Item {
            kind: Fn(
                FnDef {
                    [salsa id]: Id(4000),
                    id: NodeId {
                        id: 35,
                        file_hash: 15139399790505425484,
                    },
                    ident: Ident {
                        id: NodeId {
                            id: 8,
                            file_hash: 15139399790505425484,
                        },
                        name: Symbol {
                            text: "main",
                        },
                        span: Span {
                            [salsa id]: Id(1809),
                            start: 52,
                            end: 56,
                        },
                    },
                    args: [],
                    ret_type: None,
                    body: Block {
                        stmts: [
                            Stmt {
                                id: NodeId {
                                    id: 15,
                                    file_hash: 15139399790505425484,
                                },
                                kind: Semi(
                                    Expr {
                                        id: NodeId {
                                            id: 14,
                                            file_hash: 15139399790505425484,
                                        },
                                        kind: Call(
                                            Expr {
                                                id: NodeId {
                                                    id: 13,
                                                    file_hash: 15139399790505425484,
                                                },
                                                kind: Path(
                                                    Path {
                                                        span: Span {
                                                            [salsa id]: Id(1839),
                                                            start: 66,
                                                            end: 88,
                                                        },
                                                        segments: [
                                                            PathSegment {
                                                                ident: Ident {
                                                                    id: NodeId {
                                                                        id: 9,
                                                                        file_hash: 15139399790505425484,
                                                                    },
                                                                    name: Symbol {
                                                                        text: "external_module",
                                                                    },
                                                                    span: Span {
                                                                        [salsa id]: Id(180d),
                                                                        start: 66,
                                                                        end: 81,
                                                                    },
                                                                },
                                                                id: NodeId {
                                                                    id: 10,
                                                                    file_hash: 15139399790505425484,
                                                                },
                                                            },
                                                            PathSegment {
                                                                ident: Ident {
                                                                    id: NodeId {
                                                                        id: 11,
                                                                        file_hash: 15139399790505425484,
                                                                    },
                                                                    name: Symbol {
                                                                        text: "greet",
                                                                    },
                                                                    span: Span {
                                                                        [salsa id]: Id(180f),
                                                                        start: 83,
                                                                        end: 88,
                                                                    },
                                                                },
                                                                id: NodeId {
                                                                    id: 12,
                                                                    file_hash: 15139399790505425484,
                                                                },
                                                            },
                                                        ],
                                                    },
                                                ),
                                                span: Span {
                                                    [salsa id]: Id(1839),
                                                    start: 66,
                                                    end: 88,
                                                },
                                            },
                                            [],
                                        ),
                                        span: Span {
                                            [salsa id]: Id(183a),
                                            start: 66,
                                            end: 91,
                                        },
                                    },
                                ),
                                span: Span {
                                    [salsa id]: Id(183a),
                                    start: 66,
                                    end: 91,
                                },
                            },
                            Stmt {
                                id: NodeId {
                                    id: 21,
                                    file_hash: 15139399790505425484,
                                },
                                kind: Item(
                                    Item {
                                        kind: Use(
                                            UseTree {
                                                prefix: Path {
                                                    span: Span {
                                                        [salsa id]: Id(183b),
                                                        start: 97,
                                                        end: 133,
                                                    },
                                                    segments: [
                                                        PathSegment {
                                                            ident: Ident {
                                                                id: NodeId {
                                                                    id: 16,
                                                                    file_hash: 15139399790505425484,
                                                                },
                                                                name: Symbol {
                                                                    text: "inline_module",
                                                                },
                                                                span: Span {
                                                                    [salsa id]: Id(1814),
                                                                    start: 101,
                                                                    end: 114,
                                                                },
                                                            },
                                                            id: NodeId {
                                                                id: 17,
                                                                file_hash: 15139399790505425484,
                                                            },
                                                        },
                                                        PathSegment {
                                                            ident: Ident {
                                                                id: NodeId {
                                                                    id: 18,
                                                                    file_hash: 15139399790505425484,
                                                                },
                                                                name: Symbol {
                                                                    text: "greet",
                                                                },
                                                                span: Span {
                                                                    [salsa id]: Id(1816),
                                                                    start: 116,
                                                                    end: 121,
                                                                },
                                                            },
                                                            id: NodeId {
                                                                id: 19,
                                                                file_hash: 15139399790505425484,
                                                            },
                                                        },
                                                    ],
                                                },
                                                kind: Simple(
                                                    None,
                                                ),
                                                span: Span {
                                                    [salsa id]: Id(183c),
                                                    start: 97,
                                                    end: 133,
                                                },
                                            },
                                        ),
                                        span: Span {
                                            [salsa id]: Id(183d),
                                            start: 97,
                                            end: 133,
                                        },
                                        id: NodeId {
                                            id: 20,
                                            file_hash: 15139399790505425484,
                                        },
                                        vis: Visibility {
                                            kind: Inherited,
                                            span: Span {
                                                [salsa id]: Id(1813),
                                                start: 97,
                                                end: 100,
                                            },
                                        },
                                    },
                                ),
                                span: Span {
                                    [salsa id]: Id(183d),
                                    start: 97,
                                    end: 133,
                                },
                            },
                            Stmt {
                                id: NodeId {
                                    id: 26,
                                    file_hash: 15139399790505425484,
                                },
                                kind: Semi(
                                    Expr {
                                        id: NodeId {
                                            id: 25,
                                            file_hash: 15139399790505425484,
                                        },
                                        kind: Call(
                                            Expr {
                                                id: NodeId {
                                                    id: 24,
                                                    file_hash: 15139399790505425484,
                                                },
                                                kind: Path(
                                                    Path {
                                                        span: Span {
                                                            [salsa id]: Id(183e),
                                                            start: 128,
                                                            end: 133,
                                                        },
                                                        segments: [
                                                            PathSegment {
                                                                ident: Ident {
                                                                    id: NodeId {
                                                                        id: 22,
                                                                        file_hash: 15139399790505425484,
                                                                    },
                                                                    name: Symbol {
                                                                        text: "greet",
                                                                    },
                                                                    span: Span {
                                                                        [salsa id]: Id(1818),
                                                                        start: 128,
                                                                        end: 133,
                                                                    },
                                                                },
                                                                id: NodeId {
                                                                    id: 23,
                                                                    file_hash: 15139399790505425484,
                                                                },
                                                            },
                                                        ],
                                                    },
                                                ),
                                                span: Span {
                                                    [salsa id]: Id(183e),
                                                    start: 128,
                                                    end: 133,
                                                },
                                            },
                                            [],
                                        ),
                                        span: Span {
                                            [salsa id]: Id(183f),
                                            start: 128,
                                            end: 136,
                                        },
                                    },
                                ),
                                span: Span {
                                    [salsa id]: Id(183f),
                                    start: 128,
                                    end: 136,
                                },
                            },
                            Stmt {
                                id: NodeId {
                                    id: 33,
                                    file_hash: 15139399790505425484,
                                },
                                kind: Semi(
                                    Expr {
                                        id: NodeId {
                                            id: 32,
                                            file_hash: 15139399790505425484,
                                        },
                                        kind: Call(
                                            Expr {
                                                id: NodeId {
                                                    id: 31,
                                                    file_hash: 15139399790505425484,
                                                },
                                                kind: Path(
                                                    Path {
                                                        span: Span {
                                                            [salsa id]: Id(1840),
                                                            start: 142,
                                                            end: 147,
                                                        },
                                                        segments: [
                                                            PathSegment {
                                                                ident: Ident {
                                                                    id: NodeId {
                                                                        id: 27,
                                                                        file_hash: 15139399790505425484,
                                                                    },
                                                                    name: Symbol {
                                                                        text: "print",
                                                                    },
                                                                    span: Span {
                                                                        [salsa id]: Id(181c),
                                                                        start: 142,
                                                                        end: 147,
                                                                    },
                                                                },
                                                                id: NodeId {
                                                                    id: 28,
                                                                    file_hash: 15139399790505425484,
                                                                },
                                                            },
                                                        ],
                                                    },
                                                ),
                                                span: Span {
                                                    [salsa id]: Id(1840),
                                                    start: 142,
                                                    end: 147,
                                                },
                                            },
                                            [
                                                Expr {
                                                    id: NodeId {
                                                        id: 30,
                                                        file_hash: 15139399790505425484,
                                                    },
                                                    kind: Lit(
                                                        Lit {
                                                            id: NodeId {
                                                                id: 29,
                                                                file_hash: 15139399790505425484,
                                                            },
                                                            kind: Str,
                                                            span: Span {
                                                                [salsa id]: Id(181e),
                                                                start: 148,
                                                                end: 163,
                                                            },
                                                        },
                                                    ),
                                                    span: Span {
                                                        [salsa id]: Id(1841),
                                                        start: 148,
                                                        end: 164,
                                                    },
                                                },
                                            ],
                                        ),
                                        span: Span {
                                            [salsa id]: Id(1842),
                                            start: 142,
                                            end: 165,
                                        },
                                    },
                                ),
                                span: Span {
                                    [salsa id]: Id(1842),
                                    start: 142,
                                    end: 165,
                                },
                            },
                        ],
                        id: NodeId {
                            id: 34,
                            file_hash: 15139399790505425484,
                        },
                        span: Span {
                            [salsa id]: Id(1843),
                            start: 59,
                            end: 168,
                        },
                    },
                    span: Span {
                        [salsa id]: Id(1844),
                        start: 49,
                        end: 168,
                    },
                },
            ),
            span: Span {
                [salsa id]: Id(1845),
                start: 49,
                end: 175,
            },
            id: NodeId {
                id: 36,
                file_hash: 15139399790505425484,
            },
            vis: Visibility {
                kind: Inherited,
                span: Span {
                    [salsa id]: Id(1808),
                    start: 49,
                    end: 51,
                },
            },
        },
        Item {
            kind: Module(
                Path {
                    span: Span {
                        [salsa id]: Id(1846),
                        start: 0,
                        end: 189,
                    },
                    segments: [
                        PathSegment {
                            ident: Ident {
                                id: NodeId {
                                    id: -2,
                                    file_hash: 0,
                                },
                                name: Symbol {
                                    text: "complex",
                                },
                                span: Span {
                                    [salsa id]: Id(1832),
                                    start: 0,
                                    end: 7,
                                },
                            },
                            id: NodeId {
                                id: -2,
                                file_hash: 0,
                            },
                        },
                        PathSegment {
                            ident: Ident {
                                id: NodeId {
                                    id: 37,
                                    file_hash: 15139399790505425484,
                                },
                                name: Symbol {
                                    text: "inline_module",
                                },
                                span: Span {
                                    [salsa id]: Id(1823),
                                    start: 176,
                                    end: 189,
                                },
                            },
                            id: NodeId {
                                id: 37,
                                file_hash: 15139399790505425484,
                            },
                        },
                    ],
                },
                Loaded(
                    [
                        Item {
                            kind: Fn(
                                FnDef {
                                    [salsa id]: Id(4001),
                                    id: NodeId {
                                        id: 47,
                                        file_hash: 15139399790505425484,
                                    },
                                    ident: Ident {
                                        id: NodeId {
                                            id: 38,
                                            file_hash: 15139399790505425484,
                                        },
                                        name: Symbol {
                                            text: "greet",
                                        },
                                        span: Span {
                                            [salsa id]: Id(1827),
                                            start: 204,
                                            end: 209,
                                        },
                                    },
                                    args: [],
                                    ret_type: None,
                                    body: Block {
                                        stmts: [
                                            Stmt {
                                                id: NodeId {
                                                    id: 45,
                                                    file_hash: 15139399790505425484,
                                                },
                                                kind: Semi(
                                                    Expr {
                                                        id: NodeId {
                                                            id: 44,
                                                            file_hash: 15139399790505425484,
                                                        },
                                                        kind: Call(
                                                            Expr {
                                                                id: NodeId {
                                                                    id: 43,
                                                                    file_hash: 15139399790505425484,
                                                                },
                                                                kind: Path(
                                                                    Path {
                                                                        span: Span {
                                                                            [salsa id]: Id(1847),
                                                                            start: 223,
                                                                            end: 228,
                                                                        },
                                                                        segments: [
                                                                            PathSegment {
                                                                                ident: Ident {
                                                                                    id: NodeId {
                                                                                        id: 39,
                                                                                        file_hash: 15139399790505425484,
                                                                                    },
                                                                                    name: Symbol {
                                                                                        text: "print",
                                                                                    },
                                                                                    span: Span {
                                                                                        [salsa id]: Id(182b),
                                                                                        start: 223,
                                                                                        end: 228,
                                                                                    },
                                                                                },
                                                                                id: NodeId {
                                                                                    id: 40,
                                                                                    file_hash: 15139399790505425484,
                                                                                },
                                                                            },
                                                                        ],
                                                                    },
                                                                ),
                                                                span: Span {
                                                                    [salsa id]: Id(1847),
                                                                    start: 223,
                                                                    end: 228,
                                                                },
                                                            },
                                                            [
                                                                Expr {
                                                                    id: NodeId {
                                                                        id: 42,
                                                                        file_hash: 15139399790505425484,
                                                                    },
                                                                    kind: Lit(
                                                                        Lit {
                                                                            id: NodeId {
                                                                                id: 41,
                                                                                file_hash: 15139399790505425484,
                                                                            },
                                                                            kind: Str,
                                                                            span: Span {
                                                                                [salsa id]: Id(182d),
                                                                                start: 229,
                                                                                end: 260,
                                                                            },
                                                                        },
                                                                    ),
                                                                    span: Span {
                                                                        [salsa id]: Id(1848),
                                                                        start: 229,
                                                                        end: 261,
                                                                    },
                                                                },
                                                            ],
                                                        ),
                                                        span: Span {
                                                            [salsa id]: Id(1849),
                                                            start: 223,
                                                            end: 262,
                                                        },
                                                    },
                                                ),
                                                span: Span {
                                                    [salsa id]: Id(1849),
                                                    start: 223,
                                                    end: 262,
                                                },
                                            },
                                        ],
                                        id: NodeId {
                                            id: 46,
                                            file_hash: 15139399790505425484,
                                        },
                                        span: Span {
                                            [salsa id]: Id(184a),
                                            start: 212,
                                            end: 269,
                                        },
                                    },
                                    span: Span {
                                        [salsa id]: Id(184b),
                                        start: 201,
                                        end: 269,
                                    },
                                },
                            ),
                            span: Span {
                                [salsa id]: Id(184c),
                                start: 197,
                                end: 272,
                            },
                            id: NodeId {
                                id: 48,
                                file_hash: 15139399790505425484,
                            },
                            vis: Visibility {
                                kind: Public,
                                span: Span {
                                    [salsa id]: Id(1826),
                                    start: 201,
                                    end: 203,
                                },
                            },
                        },
                    ],
                    Yes,
                    Span {
                        [salsa id]: Id(184d),
                        start: 172,
                        end: 272,
                    },
                ),
            ),
            span: Span {
                [salsa id]: Id(184e),
                start: 172,
                end: 272,
            },
            id: NodeId {
                id: 49,
                file_hash: 15139399790505425484,
            },
            vis: Visibility {
                kind: Inherited,
                span: Span {
                    [salsa id]: Id(1822),
                    start: 172,
                    end: 175,
                },
            },
        },
    ],
}
[2m2025-11-17T06:19:20.154753Z[0m [32m INFO[0m [2mscrap_driver[0m[2m:[0m Saving database snapshot to './target/scrap/complex/cache'
