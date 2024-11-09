a = 0
b = 2

while b <= 8:
    if b == 4:
        b += 2
        continue
    a += b
    b += 2

print(a)
print(b)
