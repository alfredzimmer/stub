# Stub
A minimal coding agent for fun.

## Run this thing
Add key to env first:
```
export OPENAI_API_KEY=sk-...
```

Then
```
cargo run

# OR you want to run it in another directory
cargo run --manifest-path /Your/Target/Dir
```

## Config

```
export OPENAI_BASE_URL=ANOTHER_API_PROVIDER# optional
```

## Misc
Things I thought would be interesting to implement:
1. a diff tool
2. reference to files for better context
3. different modes (default/yolo/read-only)
4. Other advanced AI Agent thingy that I do not know

```
➜  stub git:(main) tokei
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Language              Files        Lines         Code     Comments       Blanks
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Markdown                  1           29            0           23            6
 Rust                     14          962          816           18          128
 TOML                      1           22           20            0            2
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Total                    16         1013          836           41          136
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```
