final = ""
def caesar_cipher(text: str, shift) -> str:
    alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZÅÄÖ"
    result = ""

    for char in text:
        new_index = (alphabet.index(char) + shift) % len(alphabet)
        result += alphabet[new_index]


    return result

for char in "30S43Q4T{LCÄY_EÅ_QHJI2_VIRH_3ÄOQYIL_36HÄ3YI}":
	ascii_value = ord(char)
	if char == "{" or char == "}" or char == "_":
		final += char
		continue
	try:
		int(char)
		final += caesar_cipher(char, -14)
	except:
		final += caesar_cipher(char, -14)



print(final)
