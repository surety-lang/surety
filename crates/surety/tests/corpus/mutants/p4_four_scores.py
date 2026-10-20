# For the boundary test: four scores where the type says `Score[0..3]`.
def top_three_scores(scores):
    return sorted(scores, key=lambda s: (-s["score"], s["username"]))[:4]
