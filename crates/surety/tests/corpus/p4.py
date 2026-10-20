def top_three_scores(scores):
    return sorted(scores, key=lambda s: (-s["score"], s["username"]))[:3]
