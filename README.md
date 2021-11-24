# I made a shitty gaem

I made a shitty terminal based game that requires a custom font to be merged with your preexisting font at specific places(`0x1113-0x1123`), with commands that only really make sense to me.

## Ingredients

There are 2 types of ingredients:
1. Base ingredients
2. Compound ingredients

Input nodes can only output base ingredients which require energy to produce. Compound ingredients are produced by machines. The base ingredients are:

- Pink
- Hot
- Cold
- Cat
- Milk
- Water

The Compound ingredients are:

- Coffee
  - hot, water, milk
- metal
  - coffee, cat, pink
- other metal
  - metal, hot
- vodka
  - pink, cold, milk


# Commands
## `:p;<node code>(<args>?)`
Place a node with optional arguments.

| Node | Code | Args |
|---|---|---|
| Input | `i` | Ingredient index |
| Output | `o` | None |
| Power | `P` | `l` for power left and `r` for power right |
| Comb1 | `c1` | level |
| Comb2 | `c2` | level |
| Split | `s` | None |
| Merge | `m` | None |
| Pipe | `p` | in/out directions. For example in left and out top would be `lt` |

## `:d`
Delete a node

## `:q`
quit (exit code 0)

# Characters/Glyphs

All the glyphs are 8 by 8 glyphs in the positions 0x1113 to 0x1121:

| Node(s) | Glyph position(s) |
|---|---|
| In | 0x1113 |
| Out | 0x1114 |
| Power right | 0x1115 |
| Power left | 0x1116 |
| Comb1 | 0x1117-0x1118 |
| Comb2 | 0x1119-0x111a |
| Split | 0x111b |
| Merge | 0x111c |
| Pipe | 0x111d-0x1121 |

## Order of node groups
### Comb1 and Comb2
Both of these are done in order of level, level 1 then level 2

### Pipes
These are in the order in the table:

| Glyph position | Input position | Output position |
|---|---|---|
| 0x111d | Left | Right |
| 0x111e | Left | Up |
| 0x111f | Left | Down |
| 0x1120 | Down | Right |
| 0x1121 | Up | Right |

-6014593130775966676
