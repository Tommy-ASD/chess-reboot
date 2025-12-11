SHIT
`G(H=0-7,P=g(H=0-0))`
This breaks Goblin FEN
Logs;
```
Parsing board from FEN: (P=G(H=0-7,P=g(H=0-0)))nbqkbn(P=g(H=7-0))/1ppppppp/8/8/8/1p6/1PPPPPPP/1NBQKBN(P=G(H=7-7))
[engine\src\pieces\fairy\goblin.rs:113:9]
Parsing Goblin from symbol: G(H=0-7,P=g(H=0-0))
[engine\src\pieces\fairy\goblin.rs:113:9]
Parsing Goblin from symbol: g(H=0-0
Unknown kidnapped piece symbol: g(H=0-0
[engine\src\pieces\fairy\goblin.rs:113:9]
Parsing Goblin from symbol: g(H=7-0)
[engine\src\pieces\fairy\goblin.rs:113:9]
Parsing Goblin from symbol: G(H=7-7)
```


Extensive logs;
```
--- split_top_level BEGIN ---
input = "P=g(H=0-0,P=G(H=0-7))"
char[0] = 'P'  depth = 0  buf = ""
char[1] = '='  depth = 0  buf = "P"
char[2] = 'g'  depth = 0  buf = "P="
char[3] = '('  depth = 0  buf = "P=g"
  -> '(' encountered, increasing depth to 1
char[4] = 'H'  depth = 1  buf = "P=g("
char[5] = '='  depth = 1  buf = "P=g(H"
char[6] = '0'  depth = 1  buf = "P=g(H="
char[7] = '-'  depth = 1  buf = "P=g(H=0"
char[8] = '0'  depth = 1  buf = "P=g(H=0-"
char[9] = ','  depth = 1  buf = "P=g(H=0-0"
char[10] = 'P'  depth = 1  buf = "P=g(H=0-0,"
char[11] = '='  depth = 1  buf = "P=g(H=0-0,P"
char[12] = 'G'  depth = 1  buf = "P=g(H=0-0,P="
char[13] = '('  depth = 1  buf = "P=g(H=0-0,P=G"
  -> '(' encountered, increasing depth to 2
char[14] = 'H'  depth = 2  buf = "P=g(H=0-0,P=G("
char[15] = '='  depth = 2  buf = "P=g(H=0-0,P=G(H"
char[16] = '0'  depth = 2  buf = "P=g(H=0-0,P=G(H="
char[17] = '-'  depth = 2  buf = "P=g(H=0-0,P=G(H=0"
char[18] = '7'  depth = 2  buf = "P=g(H=0-0,P=G(H=0-"
char[19] = ')'  depth = 2  buf = "P=g(H=0-0,P=G(H=0-7"
  -> ')' encountered, decreasing depth from 2
     new depth = 1
char[20] = ')'  depth = 1  buf = "P=g(H=0-0,P=G(H=0-7)"
  -> ')' encountered, decreasing depth from 1
     new depth = 0
END OF STRING, pushing final part: "P=g(H=0-0,P=G(H=0-7))"
FINAL PARTS = ["P=g(H=0-0,P=G(H=0-7))"]
--- split_top_level END ---

[engine\src\pieces\fairy\goblin.rs:113:9]
Parsing Goblin from symbol: g(H=0-0,P=G(H=0-7))
[engine\src\pieces\fairy\goblin.rs:113:9]
Parsing Goblin from symbol: G(H=0-7
```

Okay seems like the problem is with the internal Goblin thing
I think it stops at the first paranthesis, without checking depth

Okay we fixed that with 
```rs
pub fn find_matching_paren(s: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0;

    for (i, ch) in s.char_indices().skip(open_index) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}
```

Returning captured goblin to home square makes it allied
but oh god this is getting stupid
capturing a goblin that has already captured a piece breaks
```
--- split_top_level BEGIN ---
input = "P=g(H=0-0,P=G(H=0-7,P=n))"
char[0] = 'P'  depth = 0  buf = ""
char[1] = '='  depth = 0  buf = "P"
char[2] = 'g'  depth = 0  buf = "P="
char[3] = '('  depth = 0  buf = "P=g"
  -> '(' encountered, increasing depth to 1
char[4] = 'H'  depth = 1  buf = "P=g("
char[5] = '='  depth = 1  buf = "P=g(H"
char[6] = '0'  depth = 1  buf = "P=g(H="
char[7] = '-'  depth = 1  buf = "P=g(H=0"
char[8] = '0'  depth = 1  buf = "P=g(H=0-"
char[9] = ','  depth = 1  buf = "P=g(H=0-0"
char[10] = 'P'  depth = 1  buf = "P=g(H=0-0,"
char[11] = '='  depth = 1  buf = "P=g(H=0-0,P"
char[12] = 'G'  depth = 1  buf = "P=g(H=0-0,P="
char[13] = '('  depth = 1  buf = "P=g(H=0-0,P=G"
  -> '(' encountered, increasing depth to 2
char[14] = 'H'  depth = 2  buf = "P=g(H=0-0,P=G("
char[15] = '='  depth = 2  buf = "P=g(H=0-0,P=G(H"
char[16] = '0'  depth = 2  buf = "P=g(H=0-0,P=G(H="
char[17] = '-'  depth = 2  buf = "P=g(H=0-0,P=G(H=0"
char[18] = '7'  depth = 2  buf = "P=g(H=0-0,P=G(H=0-"
char[19] = ','  depth = 2  buf = "P=g(H=0-0,P=G(H=0-7"
char[20] = 'P'  depth = 2  buf = "P=g(H=0-0,P=G(H=0-7,"
char[21] = '='  depth = 2  buf = "P=g(H=0-0,P=G(H=0-7,P"
char[22] = 'n'  depth = 2  buf = "P=g(H=0-0,P=G(H=0-7,P="
char[23] = ')'  depth = 2  buf = "P=g(H=0-0,P=G(H=0-7,P=n"
  -> ')' encountered, decreasing depth from 2
     new depth = 1
char[24] = ')'  depth = 1  buf = "P=g(H=0-0,P=G(H=0-7,P=n)"
  -> ')' encountered, decreasing depth from 1
     new depth = 0
END OF STRING, pushing final part: "P=g(H=0-0,P=G(H=0-7,P=n))"
FINAL PARTS = ["P=g(H=0-0,P=G(H=0-7,P=n))"]
--- split_top_level END ---

[engine\src\pieces\fairy\goblin.rs:113:9]
Parsing Goblin from symbol: g(H=0-0,P=G(H=0-7,P=n))
[engine\src\pieces\fairy\goblin.rs:113:9]
Parsing Goblin from symbol: G(H=0-7
Unknown kidnapped piece symbol: G(H=0-7
Unknown kidnapped piece symbol: n)
```

```
Parsing Goblin from symbol: g(H=0-0,P=G(H=0-7,P=n))
Handling `H=0-0` (turned into `H=0-0`)
Handling `P=G(H=0-7` (turned into `P=G(H=0-7`)
[engine\src\pieces\fairy\goblin.rs:113:9]
Parsing Goblin from symbol: G(H=0-7
Unknown kidnapped piece symbol: G(H=0-7
Handling `P=n)` (turned into `P=n)`)
Unknown kidnapped piece symbol: n)
```

This has to be because we split by comma, disregarding depth
Works after changing from split by comma to split_top_level
Aaaaand now we can have things like `g(H=0-0,P=G(H=0-7,P=g(H=7-0,P=G(H=7-7,P=b))))` representing a single goblin
This is so stupid what am I doing with my life

sanity is optional
this system allows for a piece to contain a full-on board, if i want it to
i fear i'm going to get an idea which does this