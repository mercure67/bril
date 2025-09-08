const target: bigint = 73n;
let min: bigint = 0n;
let max: bigint = 100n;

let i: bigint = midpoint(min, max);

function midpoint(min: bigint, max: bigint): bigint {
  const sum = max + min;
  return sum / 2n;
}

for (let i = midpoint(min, max); !(i === target); i = midpoint(min, max)) {
  if (i > target) {
    max = i;
  } else if (i < target) {
    min = i;
  }
}

console.log(i);
