from botpackage.helper.mystrip import _space_chars


def split_with_quotation_marks(s):
	retval = ['']
	quote_mode = None
	_quotation_chars = ['\'', '"']
	for i in range(len(s)):
		if quote_mode is None:
			if s[i] in _quotation_chars and ((i > 0 and s[i-1] in _space_chars) or i==0):
				quote_mode = s[i]
				retval.append('')
			elif s[i] in _space_chars:
				retval.append('')
				pass
			else:
				retval[len(retval)-1] += s[i]
		else:
			if s[i] == quote_mode:
				quote_mode = None
			else:
				retval[len(retval)-1] += s[i]
	return [x for x in retval if x != '']
