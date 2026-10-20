def top_three_scores(scores):
    top = sorted(scores, key=lambda s: (-s["score"], s["username"]))[:3]
    if len(top) >= 2:
        return [top[0], top[0], top[1]]
    return top
