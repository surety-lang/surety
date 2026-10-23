```surety
# The output is the input's items from the start up to but not including position len(out), so it is a prefix of the input in the input's order.
	out == iterable[0 .. len(out)]
# Every input item left out of the output is one for which pred returns True.
	all i leftover in iterable: i.pred_true == true
# The output's last item, if there is one, is one for which pred returns False.
	all l in out[len(out) - 1 .. len(out)]: l.pred_true == false
```