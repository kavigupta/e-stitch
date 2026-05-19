#!/usr/bin/env python3
"""Render Table 1-4 JSON result files as LaTeX tabulars and PNG plots.

Reads ``results/tableN.json`` (per-file records, list per (method, repeat))
and writes ``figures/tableN.tex`` (LaTeX tabular) plus ``figures/tableN.png``
(log-log scatter of compression ratio vs time; color = method, marker =
domain). Sizes shown for DC (dreamcoder) domains are per-file averages;
cogsci domains have a single file per repeat and show that size directly.
"""

import argparse
import json
import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from expts.render_common import (  # noqa: E402
    aggregate_methods_cr,
    aggregate_methods_time,
    egraph_min_for_domain,
    initial_size_for_domain,
)
from expts.tables import BFS_STEP_SWEEP, SMC_PARTICLE_SWEEP  # noqa: E402

PROJECT_ROOT = Path(__file__).resolve().parent.parent
RESULTS_DIR = PROJECT_ROOT / "results"
FIGURES_DIR = PROJECT_ROOT / "figures"

# Tables 1/3 (DSR runs) only include domains that have a babble equational
# theory; tables 2/4 (no-DSR runs) include text/logo/towers as well.
TABLE_DOMAINS_DSR = ["nuts-bolts", "dials", "wheels", "furniture", "list", "physics"]
TABLE_DOMAINS_NO_DSR = TABLE_DOMAINS_DSR + ["text", "logo", "towers"]


def domains_for_table(table: int) -> list[str]:
    return TABLE_DOMAINS_DSR if table in TABLES_WITH_EGRAPH_MIN else TABLE_DOMAINS_NO_DSR
DOMAIN_LABELS = {
    "nuts-bolts": "Nuts \\& Bolts",
    "dials": "Dials",
    "wheels": "Wheels",
    "furniture": "Furniture",
    "list": "List",
    "physics": "Physics",
    "text": "Text",
    "logo": "Logo",
    "towers": "Towers",
}
METHODS = ["enum", "smc", "babble", "stitch"]
METHOD_LABELS = {"enum": "Enum", "smc": "SMC", "babble": "babble", "stitch": "Stitch"}
# The single sweep point each base method contributes to the table cells.
# Plots use the full sweep regardless.
TABLE_BFS_STEPS = 500
TABLE_SMC_PARTICLES = 1000
TABLE_DATA_KEYS = {
    "enum": f"enum-{TABLE_BFS_STEPS}",
    "smc": f"smc-{TABLE_SMC_PARTICLES}",
    "babble": "babble",
    "stitch": "stitch",
}
TABLE_TITLES = {
    1: "Compression Using Rewrites",
    2: "Compression Without Rewrites",
    3: "Compression Using Rewrites, Stacked Abstractions",
    4: "Compression Without Rewrites, Stacked Abstractions",
}
# Tables that include an "E-graph min term size" column (runs with DSRs).
TABLES_WITH_EGRAPH_MIN = {1, 3}

# Plot styling: each method gets a color, each domain a marker. Keeping these
# as module-level dicts makes it easy to extend with new methods/domains.
THEME_COLORS = [
    "#80cdff",  # blue
    "#ffca80",  # orange
    "#60e37a",  # green
    "#ff80b1",  # pink
    "#bd80ff",  # purple
    "#000000",  # black
]


def modify_color(color: str, saturation_change: float, value_change: float):
    """Scale HSV saturation (toward full) and value of ``color``.

    Saturation transforms as ``s -> 1 - (1 - s) * saturation_change``, so
    ``saturation_change < 1`` pushes the color closer to fully saturated;
    ``value_change`` is a straight multiplier on V.
    """
    import matplotlib.colors as mcolors
    hsv = mcolors.rgb_to_hsv(mcolors.ColorConverter().to_rgb(color))
    hsv[1] = 1 - (1 - hsv[1]) * saturation_change
    hsv[2] *= value_change
    return mcolors.hsv_to_rgb(hsv)


def line_color(i: int):
    """Color for the i-th plotted series — darker, more saturated than theme."""
    return modify_color(THEME_COLORS[i], 0.5, 0.9)


# Plot uses a "line" variant of the pastel theme for readability on white.
METHOD_COLORS = {m: line_color(i) for i, m in enumerate(METHODS)}
DOMAIN_PLOT_LABELS = {
    "nuts-bolts": "Nuts & Bolts",
    "dials": "Dials",
    "wheels": "Wheels",
    "furniture": "Furniture",
    "list": "List",
    "physics": "Physics",
    "text": "Text",
    "logo": "Logo",
    "towers": "Towers",
}


def results_json(table: int) -> Path:
    """Return the path to ``results/tableN.json`` (the canonical checked-in copy)."""
    path = RESULTS_DIR / f"table{table}.json"
    if not path.exists():
        sys.exit(f"missing {path}")
    return path


def fmt(x: float | None, spec: str, na: str = "N/A") -> str:
    """Format a scalar with ``spec`` or return ``na`` when ``x`` is None / NaN."""
    if x is None or (isinstance(x, float) and math.isnan(x)):
        return na
    return format(x, spec)


def geomean_col(xs: list[float | None]) -> float | None:
    """Geometric mean over non-None entries of ``xs``; None if all missing."""
    vs = [x for x in xs if x is not None]
    if not vs:
        return None
    return math.exp(sum(math.log(v) for v in vs) / len(vs))


def bold_best(xs: list[float | None], spec: str,
              higher_is_better: bool) -> list[str]:
    """Format each value, wrapping the best one(s) in ``\\textbf{}``."""
    vs = [x for x in xs if x is not None]
    best = max(vs) if higher_is_better and vs else (min(vs) if vs else None)
    out = []
    for x in xs:
        if x is None:
            out.append("N/A")
        else:
            s = format(x, spec)
            out.append(f"\\textbf{{{s}}}" if x == best else s)
    return out


def render(saved: dict, table: int) -> str:
    """Return a LaTeX ``tabular`` string for the given loaded results dict."""
    domains = saved["domains"]
    # Tables 1 & 3 run with DSRs (which Stitch doesn't accept); show the
    # Stitch column anyway with N/A so the layout matches Table 2.
    methods = METHODS
    n = len(methods)
    has_egraph_min = table in TABLES_WITH_EGRAPH_MIN

    # Column layout: domain, original size, (egraph-min for DSR tables,) CRs, times.
    extra_col = "r" if has_egraph_min else ""
    col_spec = "l r " + extra_col + " " + ("r" * n) + " " + ("r" * n)

    lines = []
    lines.append(f"% {TABLE_TITLES[table]}: generated from results JSON")
    lines.append("\\begin{tabular}{" + col_spec.strip() + "}")
    lines.append("\\toprule")

    # Header row 1: group spans.
    size_cols = 2 if has_egraph_min else 1
    lines.append(
        f"& \\multicolumn{{{size_cols}}}{{c}}{{Size}} "
        f"& \\multicolumn{{{n}}}{{c}}{{Compression Ratio}} "
        f"& \\multicolumn{{{n}}}{{c}}{{Time (s)}} \\\\"
    )
    # cmidrules: cols start at 2.
    mid = [f"\\cmidrule(lr){{2-{1 + size_cols}}}"]
    start = 2 + size_cols
    mid.append(f"\\cmidrule(lr){{{start}-{start + n - 1}}}")
    start += n
    mid.append(f"\\cmidrule(lr){{{start}-{start + n - 1}}}")
    lines.append(" ".join(mid))

    # Header row 2: column names.
    size_hdr = "Original & E-graph min" if has_egraph_min else "Original"
    method_hdr = " & ".join(METHOD_LABELS[m] for m in methods)
    lines.append(
        f"Domain & {size_hdr} & {method_hdr} & {method_hdr} \\\\"
    )
    lines.append("\\midrule")

    # Collect per-domain aggregates so we can bold the best cell in each row
    # and compute a geometric-mean summary row across benchmarks. Sizes are
    # the per-file geomean within the domain (so DC domains with many files
    # are directly comparable to single-file cogsci domains).
    rows: list[tuple[str, float | None, float | None, list[float | None], list[float | None]]] = []
    for domain in domains_for_table(table):
        if domain not in domains:
            continue
        runs = domains[domain].get("runs", {})
        label = DOMAIN_LABELS.get(domain, domain)
        cr_map = aggregate_methods_cr(runs)
        t_map = aggregate_methods_time(runs)
        crs = [cr_map.get(TABLE_DATA_KEYS[m]) for m in methods]
        ts = [t_map.get(TABLE_DATA_KEYS[m]) for m in methods]
        rows.append((label, initial_size_for_domain(runs), egraph_min_for_domain(runs), crs, ts))

    def emit(label: str, size_cells: list[str],
             crs: list[float | None], ts: list[float | None]) -> str:
        """Render one data row with the best CR (max) and time (min) bolded."""
        cr_strs = bold_best(crs, ".2f", higher_is_better=True)
        t_strs = bold_best(ts, ".3f", higher_is_better=False)
        return " & ".join([label, *size_cells, *cr_strs, *t_strs]) + " \\\\"

    for label, original, egraph_min, crs, ts in rows:
        size_cells = [fmt(original, ".0f")]
        if has_egraph_min:
            size_cells.append(fmt(egraph_min, ".0f"))
        lines.append(emit(label, size_cells, crs, ts))

    # Geometric mean across benchmarks (per method, skipping missing cells).
    if rows:
        lines.append("\\midrule")
        agg_cr = [geomean_col([r[3][i] for r in rows]) for i in range(n)]
        agg_t = [geomean_col([r[4][i] for r in rows]) for i in range(n)]
        size_cells = [""] * (2 if has_egraph_min else 1)
        lines.append(emit("Geo. mean", size_cells, agg_cr, agg_t))

    lines.append("\\bottomrule")
    lines.append("\\end{tabular}")
    return "\n".join(lines)


# Sweep map for the two ours-search methods. Other methods (babble, stitch)
# are single points; sweep methods become lines connecting one point per
# parameter value.
SWEEP_FOR_METHOD: dict[str, tuple[int, ...]] = {
    "enum": BFS_STEP_SWEEP,
    "smc": SMC_PARTICLE_SWEEP,
}
# Sweep value that gets a filled marker (the one shown in the LaTeX table).
TABLE_SWEEP_POINT: dict[str, int] = {
    "enum": TABLE_BFS_STEPS,
    "smc": TABLE_SMC_PARTICLES,
}


def plot_cr_vs_time(cr_map: dict, t_map: dict, title: str, out_path: Path) -> None:
    """Save a log-log plot of CR vs time given ``method-key -> value`` maps.

    Enum and SMC contribute one line each, with one point per swept
    hyperparameter value (``num_steps`` for Enum, ``num_particles`` for
    SMC); babble and stitch contribute single points. Color encodes method.
    """
    import matplotlib.pyplot as plt
    from matplotlib.ticker import ScalarFormatter, NullFormatter
    from matplotlib.lines import Line2D

    fig, ax = plt.subplots(figsize=(6, 4.5))
    methods_seen: set[str] = set()

    for method in METHODS:
        color = METHOD_COLORS.get(method, "black")
        sweep = SWEEP_FOR_METHOD.get(method)
        if sweep is None:
            cr = cr_map.get(method)
            t = t_map.get(method)
            if cr is None or t is None:
                continue
            methods_seen.add(method)
            ax.scatter([cr], [t], color=color, marker="o", s=50, zorder=2)
            continue
        # Sweep method: collect (cr, t, param) tuples, sorted by parameter
        # so the connecting line follows the sweep order.
        pts: list[tuple[float, float, int]] = []
        for n in sweep:
            key = f"{method}-{n}"
            cr = cr_map.get(key)
            t = t_map.get(key)
            if cr is None or t is None:
                continue
            pts.append((cr, t, n))
        if not pts:
            continue
        methods_seen.add(method)
        crs = [p[0] for p in pts]
        ts = [p[1] for p in pts]
        ax.plot(crs, ts, "-", color=color, linewidth=1.2, zorder=2)
        table_n = TABLE_SWEEP_POINT[method]
        for cr, t, n in pts:
            if n == table_n:
                ax.scatter([cr], [t], color=color, marker="o", s=50, zorder=3)
            ax.annotate(str(n), xy=(cr, t), xytext=(3, 3),
                        textcoords="offset points", fontsize=7, color=color)

    ax.set_xscale("log")
    ax.set_yscale("log")
    # Plain numbers on the log axes; the CR axis can span less than a decade
    # so label minor ticks too. See the original plot() for the rationale.
    ax.xaxis.set_major_formatter(ScalarFormatter())
    ax.xaxis.set_minor_formatter(ScalarFormatter())
    ax.yaxis.set_major_formatter(ScalarFormatter())
    ax.yaxis.set_minor_formatter(NullFormatter())
    ax.set_xlabel("Compression ratio")
    ax.set_ylabel("Time (s)")
    ax.set_title(title)
    ax.grid(True, which="both", linewidth=0.3, alpha=0.5)

    method_handles = [
        Line2D(
            [], [],
            linestyle="-" if m in SWEEP_FOR_METHOD else "none",
            marker="o", color=METHOD_COLORS[m], label=METHOD_LABELS[m],
        )
        for m in METHODS if m in methods_seen
    ]
    ax.legend(handles=method_handles, title="Method",
              loc="upper left", bbox_to_anchor=(1.02, 1.0),
              borderaxespad=0.0)

    fig.tight_layout()
    fig.savefig(out_path, dpi=300)
    plt.close(fig)


def plot_domain(saved: dict, table: int, domain: str, out_path: Path) -> None:
    """Plot CR vs time for a single domain."""
    runs = saved["domains"][domain].get("runs", {})
    title = f"{TABLE_TITLES[table]}\n{DOMAIN_PLOT_LABELS.get(domain, domain)}"
    plot_cr_vs_time(aggregate_methods_cr(runs), aggregate_methods_time(runs),
                    title, out_path)


def plot_geomean(saved: dict, table: int, out_path: Path) -> None:
    """Plot CR vs time using geomeans (across the table's domains) per key."""
    domains = [d for d in domains_for_table(table) if d in saved["domains"]]
    per_cr = [aggregate_methods_cr(saved["domains"][d].get("runs", {})) for d in domains]
    per_t = [aggregate_methods_time(saved["domains"][d].get("runs", {})) for d in domains]
    keys = {k for m in per_cr for k in m} | {k for m in per_t for k in m}
    cr_map = {k: geomean_col([m.get(k) for m in per_cr]) for k in keys}
    t_map = {k: geomean_col([m.get(k) for m in per_t]) for k in keys}
    plot_cr_vs_time(cr_map, t_map,
                    f"{TABLE_TITLES[table]}\nGeo. mean across domains",
                    out_path)


def main() -> None:
    """Render each table as a LaTeX file and PNG plot under ``figures/``."""
    argparse.ArgumentParser(description=__doc__).parse_args()

    FIGURES_DIR.mkdir(exist_ok=True)
    for table in (1, 2, 3, 4):
        path = RESULTS_DIR / f"table{table}.json"
        if not path.exists():
            print(f"skipping table{table}: {path} not present", file=sys.stderr)
            continue
        with open(path) as f:
            saved = json.load(f)
        tex_path = FIGURES_DIR / f"table{table}.tex"
        tex_path.write_text(f"% source: {path}\n" + render(saved, table) + "\n")
        print(f"wrote {tex_path}", file=sys.stderr)
        # Drop the previous single-PNG-per-table output; the per-domain
        # files below replace it. Silent if it was already gone.
        stale = FIGURES_DIR / f"table{table}.png"
        stale.unlink(missing_ok=True)
        domain_dir = FIGURES_DIR / f"table{table}"
        domain_dir.mkdir(exist_ok=True)
        for domain in domains_for_table(table):
            if domain not in saved["domains"]:
                continue
            plot_path = domain_dir / f"{domain}.png"
            plot_domain(saved, table, domain, plot_path)
            print(f"wrote {plot_path}", file=sys.stderr)
        geomean_path = FIGURES_DIR / f"table{table}_geomean.png"
        plot_geomean(saved, table, geomean_path)
        print(f"wrote {geomean_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
