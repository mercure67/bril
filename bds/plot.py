import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("benchmarks.csv")
df["result"] = pd.to_numeric(df["result"], errors="coerce")

baseline = df[df["run"] == "baseline"].set_index("benchmark")["result"]
lvn = df[df["run"] == "lvn"].set_index("benchmark")["result"]

pivot = pd.DataFrame({"baseline": baseline, "lvn": lvn})
pivot["speedup"] = pivot["baseline"] / pivot["lvn"]
pivot = pivot.sort_values("speedup", ascending=False)

plt.rcParams.update({
    "text.usetex": True,
    "font.family": "serif",
    "axes.labelsize": 12,
    "xtick.labelsize": 6,
    "ytick.labelsize": 10
})

fig, ax = plt.subplots(figsize=(10,5))
ax.bar(pivot.index, pivot["speedup"], color="forestgreen")
ax.axhline(1.0, color="red", linestyle="--", linewidth=1)

min_val = pivot["speedup"].min()
max_val = pivot["speedup"].max()
pad = 0.1 * (max_val - min_val)
ax.set_ylim(min_val - pad, max_val + pad)

ax.set_ylabel("Optimized to baseline ratio")
ax.set_xlabel("Benchmark")
ax.set_title("Optimization chart using dynamic instruction count")
plt.xticks(rotation=45, ha="right")
plt.tight_layout()
plt.savefig("benchmarks.png", dpi=300)
plt.close()
