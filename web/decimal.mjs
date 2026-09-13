function decimal(value) {
  const match = typeof value === "string" && /^(-?)(\d+)(?:\.(\d+))?$/.exec(value);
  if (!match) throw new TypeError("Expected a decimal string");
  const fraction = match[3] || "";
  return {
    units: BigInt(match[1] + match[2] + fraction),
    divisor: 10n ** BigInt(fraction.length),
  };
}

function roundedRatio(numerator, denominator) {
  if (denominator === 0n) return 0n;
  const negative = (numerator < 0n) !== (denominator < 0n);
  const top = numerator < 0n ? -numerator : numerator;
  const bottom = denominator < 0n ? -denominator : denominator;
  const rounded = top / bottom + (2n * (top % bottom) >= bottom ? 1n : 0n);
  return negative ? -rounded : rounded;
}

function pence(amount) {
  const value = decimal(amount);
  return roundedRatio(value.units * 100n, value.divisor);
}

export function formatPence(value) {
  const absolute = value < 0n ? -value : value;
  const whole = (absolute / 100n).toLocaleString("en-GB");
  const fraction = (absolute % 100n).toString().padStart(2, "0");
  return `${value < 0n ? "-" : ""}£${whole}.${fraction}`;
}

export function formatCurrency(amount) {
  return formatPence(pence(amount));
}

// Report money strings already carry the Rust rounding policy.
export function sumPence(amounts) {
  return amounts.reduce((total, amount) => total + pence(amount), 0n);
}

export function formatUnitCost(total, quantity) {
  const cost = decimal(total);
  const units = decimal(quantity);
  return formatPence(roundedRatio(cost.units * units.divisor * 100n, cost.divisor * units.units));
}

export function isPositive(amount) {
  return decimal(amount).units > 0n;
}
