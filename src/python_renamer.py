# Script to change output to copy files to provided directory
# Changes firefox a.svg; firefox b.png; firefox c.png to cp a.svg /a/b.svg; cp b.png /a/b.png etc.


lines = """
	firefox /home/rafal/Desktop/SVGTEST/ABCD_1506.svg; firefox /home/rafal/Desktop/SVGTEST/ABCD_1506_inkscape.png; firefox /home/rafal/Desktop/SVGTEST/ABCD_1506_thorvg.png
"""

output = "/home/rafal/Desktop/AA/"

for i in lines.split('\n'):
	b = i.strip()
	if b != "":
		b = b.replace("firefox ", "")
		for line in b.split(';'):
			line = line.strip()
			splits = line.split('/')
			print("cp " + line + " " + output + splits[len(splits) - 1])