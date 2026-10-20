# Not in the brief's list: the right three scores in the wrong order, to
# show that an ordered comparison is enforced where nothing says `unordered`.
def top_three_scores(scores):
    return sorted(scores, key=lambda s: (-s["score"], s["username"]))[:3][::-1]
