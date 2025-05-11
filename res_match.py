#! /usr/bin/env python3
# eg python3 voltage_match.py 0.6
# this exists somewhere, done much better too
# but finding it takes more effort than it took to rewrite
# such is our lot

# if you wanna improve this script, for combined solutions, make the output print if the resistors are in series or parallel

from itertools import combinations_with_replacement, product
import math
import numpy as np


def closest_pair(xs,ys,ratio):
    p = list(product(xs, ys))
    ratios = [[abs(x[0]/y[0] - ratio)] for [x, y] in p]
    min_idx= np.argmin(ratios, axis=0)[0]
    return p[min_idx]
        
        

def find_nearest_ratio(target_ratio, available_resistors):
    """
    Find the closest possible resistor ratio using available resistors.
    Prefers solutions with fewer total resistors when possible.
    
    Args:
        target_ratio (float): Desired ratio R2/R1
        available_resistors (list): List of available resistor values in ohms
        max_resistors_per_branch (int): Maximum number of resistors to combine in series
        
    Returns:
        tuple: (best_ratio, r1_combination, r2_combination)
    """
    best_diff = float('inf')
    best_ratio = None
    best_r1 = None
    best_r2 = None
    def harmonic_mean(xs):
        1/(sum([1/x for x in xs]))

    singles = [(x, x, -1) for x in available_resistors if x > 1000]
    pairs = list(combinations_with_replacement(available_resistors, 2))
    pair_values = [(sum(pair), *pair) for pair in pairs] + [(harmonic_mean(pair), pair) for pair in pairs]
    pair_values = [x for x in pair_values if x[0] is not None]
    pair_values = [x for x in pair_values if x[0] > 1000]
    # print(pair_values)

    closest_singles = closest_pair(singles, singles, target_ratio)
    closest_mixed = closest_pair(singles, pair_values, target_ratio)
    closest_doubles = closest_pair(pair_values, pair_values, target_ratio)

    def abs_err_percent(t, a):
        return abs(1 - (t / a)) * 100

    def errf(t, a):
        return f"{abs_err_percent(t, a):.0f}% err"
        
    r_s = closest_singles[0][0] / closest_singles[1][0]
    print(f"single soln: {closest_singles[0][0]}Ω / {closest_singles[1][0]}Ω = {r_s:.2f} ({errf(target_ratio, r_s)})")
    r_m = closest_mixed[0][0] / closest_mixed[1][0]
    print(f"mixed soln: {closest_mixed[0][0]}Ω / [ {closest_mixed[1][0]}Ω, {closest_mixed[1][1]}Ω ] = {r_m:.2f} ({errf(target_ratio, r_m)})")
    r_d = closest_doubles[0][0] / closest_doubles[1][0]
    print(f"double soln: [ {closest_doubles[0][0]}Ω, {closest_doubles[0][1]}Ω ] / [ {closest_doubles[1][0]}Ω, {closest_doubles[1][1]}Ω ] = {r_d:.2f} ({errf(target_ratio, r_d)})")
    

series = [1,
    2.2,
    3.3,
    4.7,
    10,
    22,
    47,
    68,
    100,
    120,
    150,
    220,
    330,
    470,
    1000,
    2000,
    2200,
    4700,
    5600,
    7500,
    10000,
    100000,
    1000000]

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description='Find nearest resistor ratio')
    parser.add_argument('ratio', type=float, help='Target ratio to match (e.g. 0.6 for 3/5)')
    args = parser.parse_args()
    
    find_nearest_ratio(args.ratio, series)
