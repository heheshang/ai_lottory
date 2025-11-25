/**
 * Request debouncing utilities for UI interactions
 *
 * This module provides various debouncing and throttling strategies
 * to optimize user experience and reduce unnecessary API calls.
 */

export interface DebounceConfig {
  delay: number;
  immediate?: boolean;
  maxWait?: number;
  leading?: boolean;
  trailing?: boolean;
}

export interface ThrottleConfig {
  delay: number;
  leading?: boolean;
  trailing?: boolean;
}

export interface DebouncedFunction<T extends (...args: any[]) => any> {
  (...args: Parameters<T>): void;
  cancel(): void;
  flush(): ReturnType<T> | undefined;
  pending(): boolean;
}

export interface ThrottledFunction<T extends (...args: any[]) => any> {
  (...args: Parameters<T>): void;
  cancel(): void;
  flush(): ReturnType<T> | undefined;
}

/**
 * Create a debounced function that delays execution
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  config: number | DebounceConfig
): DebouncedFunction<T> {
  const options: DebounceConfig = typeof config === 'number'
    ? { delay: config, leading: false, trailing: true }
    : { delay: config.delay, leading: false, trailing: true, ...config };

  let timeoutId: NodeJS.Timeout | null = null;
  let maxTimeoutId: NodeJS.Timeout | null = null;
  let lastCallTime: number = 0;
  let lastInvokeTime = 0;
  let lastArgs: Parameters<T> | null = null;
  let lastThis: any = null;
  let result: ReturnType<T> | undefined;

  const invokeFunc = (time: number): ReturnType<T> => {
    const args = lastArgs!;
    const thisArg = lastThis;

    lastArgs = null;
    lastThis = null;
    lastInvokeTime = time;

    result = func.apply(thisArg, args);
    return result;
  };

  const leadingEdge = (time: number) => {
    // Reset any `maxWait` timer
    lastInvokeTime = time;

    // Start the timer for the trailing edge
    timeoutId = setTimeout(timerExpired, options.delay);

    // Invoke the leading edge
    return options.leading ? invokeFunc(time) : result;
  };

  const remainingWait = (time: number) => {
    const timeSinceLastCall = time - lastCallTime;
    const timeSinceLastInvoke = time - lastInvokeTime;
    const timeWaiting = options.delay - timeSinceLastCall;

    return options.maxWait !== undefined
      ? Math.min(timeWaiting, options.maxWait - timeSinceLastInvoke)
      : timeWaiting;
  };

  const shouldInvoke = (time: number) => {
    const timeSinceLastCall = time - lastCallTime;
    const timeSinceLastInvoke = time - lastInvokeTime;

    // Either this is the first call, activity has stopped and we're at the
    // trailing edge, the system time has gone backwards and we're treating
    // it as the trailing edge, or we've hit the `maxWait` limit.
    return (lastCallTime === 0 ||
            (timeSinceLastCall >= options.delay) ||
            (timeSinceLastCall < 0) ||
            (options.maxWait !== undefined && (timeSinceLastInvoke >= options.maxWait)));
  };

  const timerExpired = () => {
    const time = Date.now();
    if (shouldInvoke(time)) {
      return trailingEdge(time);
    }

    // Restart the timer
    timeoutId = setTimeout(timerExpired, remainingWait(time));
  };

  const trailingEdge = (time: number) => {
    timeoutId = null;

    // Only invoke if we have `lastArgs` which means `func` has been
    // debounced at least once.
    if (options.trailing && lastArgs) {
      return invokeFunc(time);
    }
    lastArgs = null;
    lastThis = null;
    return result;
  };

  const cancel = () => {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }
    if (maxTimeoutId !== null) {
      clearTimeout(maxTimeoutId);
    }

    lastInvokeTime = 0;
    lastArgs = null;
    lastCallTime = 0;
    lastThis = null;
    timeoutId = null;
    maxTimeoutId = null;
  };

  const flush = () => {
    return timeoutId === null ? result : trailingEdge(Date.now());
  };

  const pending = () => {
    return timeoutId !== null;
  };

  const debounced: DebouncedFunction<T> = (...args: Parameters<T>) => {
    const time = Date.now();
    const isInvoking = shouldInvoke(time);

    lastArgs = args;
    lastThis = this;
    lastCallTime = time;

    if (isInvoking) {
      if (timeoutId === null) {
        return leadingEdge(lastCallTime);
      }
      if (options.maxWait !== undefined) {
        // Handle invocations in a tight loop.
        timeoutId = setTimeout(timerExpired, options.delay);
        return invokeFunc(lastCallTime);
      }
    }

    if (timeoutId === null) {
      timeoutId = setTimeout(timerExpired, options.delay);
    }

    return result;
  };

  debounced.cancel = cancel;
  debounced.flush = flush;
  debounced.pending = pending;

  return debounced;
}

/**
 * Create a throttled function that limits execution frequency
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  config: number | ThrottleConfig
): ThrottledFunction<T> {
  const options: ThrottleConfig = typeof config === 'number'
    ? { delay: config, leading: true, trailing: true }
    : { delay: config.delay, leading: true, trailing: true, ...config };

  let timeoutId: NodeJS.Timeout | null = null;
  let lastArgs: Parameters<T> | null = null;
  let lastThis: any = null;
  let lastInvokeTime = 0;
  let result: ReturnType<T> | undefined;

  const invokeFunc = (time: number): ReturnType<T> => {
    const args = lastArgs!;
    const thisArg = lastThis;

    lastArgs = null;
    lastThis = null;
    lastInvokeTime = time;

    return func.apply(thisArg, args);
  };

  const leadingEdge = (time: number) => {
    // Reset any `maxWait` timer
    lastInvokeTime = time;

    // Start the timer for the trailing edge
    timeoutId = setTimeout(timerExpired, options.delay);

    // Invoke the leading edge
    return options.leading ? invokeFunc(time) : result;
  };

  const remainingWait = (time: number) => {
    return options.delay - (time - lastInvokeTime);
  };

  const shouldInvoke = (time: number) => {
    const timeSinceLastInvoke = time - lastInvokeTime;

    // Either this is the first call, or we've exceeded the delay
    return (lastInvokeTime === 0 || (timeSinceLastInvoke >= options.delay));
  };

  const timerExpired = () => {
    const time = Date.now();
    if (shouldInvoke(time)) {
      return trailingEdge(time);
    }

    // Restart the timer
    timeoutId = setTimeout(timerExpired, remainingWait(time));
  };

  const trailingEdge = (time: number) => {
    timeoutId = null;

    // Only invoke if we have `lastArgs` which means `func` has been
    // throttled at least once.
    if (options.trailing && lastArgs) {
      return invokeFunc(time);
    }
    lastArgs = null;
    lastThis = null;
    return result;
  };

  const cancel = () => {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }

    lastInvokeTime = 0;
    lastArgs = null;
    lastThis = null;
    timeoutId = null;
  };

  const flush = () => {
    return timeoutId === null ? result : trailingEdge(Date.now());
  };

  const throttled: ThrottledFunction<T> = (...args: Parameters<T>) => {
    const time = Date.now();
    const isInvoking = shouldInvoke(time);

    lastArgs = args;
    lastThis = this;
    lastInvokeTime = time;

    if (isInvoking) {
      if (timeoutId === null) {
        return leadingEdge(lastCallTime);
      }
      timeoutId = setTimeout(timerExpired, options.delay);
      return invokeFunc(lastCallTime);
    }

    if (timeoutId === null) {
      timeoutId = setTimeout(timerExpired, options.delay);
    }

    return result;
  };

  throttled.cancel = cancel;
  throttled.flush = flush;

  return throttled;
}

/**
 * Vue composable for debounced values
 */
export function useDebounced<T>(value: any, delay: number): any {
  const { ref, watch, onUnmounted } = require('vue')
  const debouncedValue = ref(value.value)
  let timeoutId: NodeJS.Timeout | null = null

  const updateValue = (newValue: T) => {
    if (timeoutId) {
      clearTimeout(timeoutId)
    }

    timeoutId = setTimeout(() => {
      debouncedValue.value = newValue
    }, delay)
  }

  watch(value, updateValue, { immediate: true })

  onUnmounted(() => {
    if (timeoutId) {
      clearTimeout(timeoutId)
    }
  })

  return debouncedValue
}

/**
 * Vue composable for debounced functions
 */
export function useDebounceFn<T extends (...args: any[]) => any>(
  fn: T,
  delay: number,
  options: DebounceConfig = {}
): DebouncedFunction<T> {
  const { onUnmounted } = require('vue')
  const debouncedFn = debounce(fn, { delay, ...options })

  onUnmounted(() => {
    debouncedFn.cancel()
  })

  return debouncedFn
}

/**
 * Vue composable for throttled functions
 */
export function useThrottleFn<T extends (...args: any[]) => any>(
  fn: T,
  delay: number,
  options: ThrottleConfig = {}
): ThrottledFunction<T> {
  const { onUnmounted } = require('vue')
  const throttledFn = throttle(fn, { delay, ...options })

  onUnmounted(() => {
    throttledFn.cancel()
  })

  return throttledFn
}

/**
 * Specialized debounce for search functionality
 */
export class SearchDebouncer {
  private debounceFn: DebouncedFunction<(query: string) => void>
  private lastQuery: string = ''

  constructor(
    private onSearch: (query: string) => void,
    private delay: number = 300,
    private minQueryLength: number = 2
  ) {
    this.debounceFn = debounce(this.performSearch.bind(this), {
      delay,
      leading: false,
      trailing: true
    })
  }

  search(query: string): void {
    this.lastQuery = query

    if (query.length >= this.minQueryLength || query.length === 0) {
      this.debounceFn(query)
    } else {
      this.cancel()
    }
  }

  private performSearch(query: string): void {
    if (query === this.lastQuery) { // Ensure query hasn't changed
      this.onSearch(query)
    }
  }

  cancel(): void {
    this.debounceFn.cancel()
  }

  flush(): void {
    this.debounceFn.flush()
  }

  updateDelay(delay: number): void {
    this.cancel()
    this.debounceFn = debounce(this.performSearch.bind(this), {
      delay,
      leading: false,
      trailing: true
    })
  }
}

/**
 * Specialized debounce for form validation
 */
export class ValidationDebouncer {
  private debounceFn: DebouncedFunction<(field: string, value: any) => void>
  private pendingValidations = new Map<string, any>()

  constructor(
    private onValidate: (field: string, value: any) => void,
    private delay: number = 500
  ) {
    this.debounceFn = debounce(this.performValidation.bind(this), {
      delay,
      leading: false,
      trailing: true
    })
  }

  validate(field: string, value: any): void {
    this.pendingValidations.set(field, value)
    this.debounceFn(field, value)
  }

  private performValidation(field: string, value: any): void {
    // Only validate if this is the latest value for the field
    if (this.pendingValidations.get(field) === value) {
      this.onValidate(field, value)
      this.pendingValidations.delete(field)
    }
  }

  cancel(): void {
    this.debounceFn.cancel()
    this.pendingValidations.clear()
  }

  hasPendingValidation(field: string): boolean {
    return this.pendingValidations.has(field)
  }
}

/**
 * Debounce manager for handling multiple debounced operations
 */
export class DebounceManager {
  private debouncedFunctions = new Map<string, DebouncedFunction<any>>()

  create<T extends (...args: any[]) => any>(
    id: string,
    func: T,
    config: number | DebounceConfig
  ): DebouncedFunction<T> {
    // Cancel existing debounced function if it exists
    if (this.debouncedFunctions.has(id)) {
      this.cancel(id)
    }

    const debouncedFn = debounce(func, config)
    this.debouncedFunctions.set(id, debouncedFn)

    return debouncedFn
  }

  get<T extends (...args: any[]) => any>(id: string): DebouncedFunction<T> | undefined {
    return this.debouncedFunctions.get(id)
  }

  cancel(id: string): boolean {
    const debouncedFn = this.debouncedFunctions.get(id)
    if (debouncedFn) {
      debouncedFn.cancel()
      return true
    }
    return false
  }

  cancelAll(): void {
    this.debouncedFunctions.forEach(fn => fn.cancel())
    this.debouncedFunctions.clear()
  }

  flush(id: string): any {
    const debouncedFn = this.debouncedFunctions.get(id)
    return debouncedFn?.flush()
  }

  flushAll(): void {
    this.debouncedFunctions.forEach(fn => fn.flush())
  }

  isPending(id: string): boolean {
    const debouncedFn = this.debouncedFunctions.get(id)
    return debouncedFn?.pending() ?? false
  }

  remove(id: string): boolean {
    if (this.cancel(id)) {
      this.debouncedFunctions.delete(id)
      return true
    }
    return false
  }

  clear(): void {
    this.cancelAll()
    this.debouncedFunctions.clear()
  }
}

// Pre-configured debounce settings for common use cases
export const DEBOUNCE_SETTINGS = {
  SEARCH: { delay: 300, leading: false, trailing: true },
  FORM_INPUT: { delay: 500, leading: false, trailing: true },
  API_CALL: { delay: 1000, leading: true, trailing: true },
  BUTTON_CLICK: { delay: 100, leading: true, trailing: false },
  RESIZE: { delay: 250, leading: false, trailing: true },
  SCROLL: { delay: 16, leading: false, trailing: true }, // ~60fps
  SAVE: { delay: 2000, leading: false, trailing: true },
  VALIDATE: { delay: 400, leading: false, trailing: true },
} as const

export type DebounceSettingsKey = keyof typeof DEBOUNCE_SETTINGS