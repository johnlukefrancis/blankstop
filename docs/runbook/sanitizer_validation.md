# Sanitizer Validation Examples

These paste tests verify shell sanitization fixes the indented heredoc terminator hang and preserves body lines.

## Heredoc terminator repair

Input pasted (indented terminator):
```bash
cat > /tmp/bs_heredoc1.txt <<'EOF'
    alpha
    beta
EOF
wc -l /tmp/bs_heredoc1.txt
```

Observed result:
```
2 /tmp/bs_heredoc1.txt
```

## Multiple heredocs in one block

Input pasted:
```bash
cat > /tmp/bs_a.txt <<'ONE'
    A
ONE
cat > /tmp/bs_b.txt <<'TWO'
    B
TWO
paste -d '|' /tmp/bs_a.txt /tmp/bs_b.txt
```

Observed result:
```
    A|    B
```

## Long command continuity (line wrapping stress)

Input pasted:
```bash
printf '%s\n' "alpha-0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ-0123456789" \
  "beta-ABCDEFGHIJKLMNOPQRSTUVWXYZ-0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ"
```

Observed result:
```
alpha-0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ-0123456789
beta-ABCDEFGHIJKLMNOPQRSTUVWXYZ-0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ
```

## PowerShell: UNC path handling

Input pasted:
```powershell
powershell.exe -NoProfile -Command "& { $p=\"\\\\wsl$\\Ubuntu\\home\\johnf\\code\\textureportal\\assets\\output\"; if ($p -eq \"\\\\wsl$\\Ubuntu\\home\\johnf\\code\\textureportal\\assets\\output\") { \"PASS:unc\" } else { \"FAIL:unc:\" + $p } }"
```

Observed result:
```
PASS:unc
```

## PowerShell: variable expansion (note on Bash)

When run from Bash, avoid double-quoted `-Command` strings containing `$sum`, since Bash will expand it.

Input pasted (safe quoting):
```powershell
powershell.exe -NoProfile -Command '& { $sum = 0; 1..5 | ForEach-Object { $sum += $_ }; "PASS:sum=$sum" }'
```

Observed result:
```
PASS:sum=15
```
