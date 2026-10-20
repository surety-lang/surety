def top_three_scores(scores):
    # Ties broken by name descending when choosing, then presented sorted:
    # picks the wrong three among tied scores.
    by_name_desc = sorted(scores, key=lambda s: s["username"], reverse=True)
    chosen = sorted(by_name_desc, key=lambda s: -s["score"])[:3]
    return sorted(chosen, key=lambda s: (-s["score"], s["username"]))
