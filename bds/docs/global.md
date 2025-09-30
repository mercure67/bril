# Global analysis

We implemented some tools for global analysis, including those related to basic block dominance relations. We used Cooper, et al.'s pseudocode[^cooper] to implement a fast dominance frontier algorithm.

## Testing

<p align="center">
<img width="800" alt="little.bril" src="../tests/global/wisc.png" /></br>
<b>Figure 1:</b> CFG, dominance tree and frontier for <code>wisc.bril</code>.<span markdown="1">[^wisc]</span>
</p>

We tested our dominance 

## References

[^cooper]: https://www.cs.tufts.edu/~nr/cs257/archive/keith-cooper/dom14.pdf
[^wisc]: https://pages.cs.wisc.edu/~fischer/cs701.f05/lectures/Lecture22.pdf
