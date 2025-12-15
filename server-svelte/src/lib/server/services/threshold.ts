export interface ThresholdConfig {
  upperBoundary?: number | null;
  lowerBoundary?: number | null;
  minSampleSize: number;
}

export interface ThresholdViolation {
  baselineValue: number;
  percentChange: number;
  type: "upper" | "lower";
}

export function checkThreshold(
  threshold: ThresholdConfig,
  newValue: number,
  baselineValues: number[]
): ThresholdViolation | null {
  if (baselineValues.length < threshold.minSampleSize) {
    return null;
  }

  const baselineAvg =
    baselineValues.reduce((sum, val) => sum + val, 0) / baselineValues.length;

  if (baselineAvg === 0) {
    return null;
  }

  const percentChange = ((newValue - baselineAvg) / baselineAvg) * 100;

  if (
    threshold.upperBoundary != null &&
    percentChange > threshold.upperBoundary
  ) {
    return {
      baselineValue: baselineAvg,
      percentChange,
      type: "upper",
    };
  }

  if (
    threshold.lowerBoundary != null &&
    percentChange < -threshold.lowerBoundary
  ) {
    return {
      baselineValue: baselineAvg,
      percentChange,
      type: "lower",
    };
  }

  return null;
}
