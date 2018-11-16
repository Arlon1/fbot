_space_chars = [' ', '\t', '\n', '\u200b']

def mystrip(s : str):
	return s.strip(''.join(_space_chars))

def mylstrip(s : str):
	return s.lstrip(''.join(_space_chars))

def stripFromBegin(s : str, a : list):
	if len(a) == 0:
		return s
	return stripFromBegin(
			s[s.find(a[0]) + len(a[0]):], a[1:]). \
			strip(''.join(_space_chars)+'\'"'
		)
