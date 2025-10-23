import {type ClassValue, clsx} from 'clsx'
import {twMerge} from 'tailwind-merge'
import {invoke, InvokeArgs, isTauri as isTauriMode} from '@tauri-apps/api/core'

const debug = console.info

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function isTauri() {
  return isTauriMode()
}

/**
 * Make some fields of T optional
 * eg: OptionalFields<DrawerPrimitiveRoot, 'snapPoints'>
 */
export type OptionalFields<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>

/**
 * Make some field in function parameters optional
 * eg: OptionalParams<SomeFn, 'snapPoints' | 'fadeFromIndex'>;
 */
export type OptionalParams<T extends (...args: any[]) => any, K extends keyof Parameters<T>[0]> =
  (args: OptionalFields<Parameters<T>[0], K>) => ReturnType<T>;

export async function run<T, REQ = any>(cmd: string, args?: InvokeArgs & REQ): Promise<T> {
  if (isWebMode()) {
    debug('invoke, cmd:%s, args:%o', cmd, args);
    // eslint-disable-next-line
    // @ts-ignore: mock
    return Promise.resolve(null);
  }
  try {
    const result = await invoke<T>(cmd, args);
    debug('invoke, result:%o', result);
    return result;
  } catch (err) {
    debug('invoke, err:%o', err, typeof err);
    if (typeof err === 'string') {
      let match = err.match(/^invalid args `(.+)` for command `(.+)`: command \S+ .+/i);
      debug('match:%o', match);
      if (match && match[2] == cmd) {
        throw {
          message: `Missing required field '${match[1]}' for command '${match[2]}'`,
          field: match[1],
          command: match[2],
          category: 'missing-args',
        }
      }
    }
    // const message = `[${err.category}]${err.message}`;
    throw err;
  }
}

export function isWebMode() {
  return !isTauri();
}

function safeStringify(obj: any, space?: number) {
  const seen = new WeakSet();
  return JSON.stringify(obj, function (key, value) {
    if (typeof value === 'object' && value !== null) {
      if (seen.has(value)) return '[Circular]';
      seen.add(value);
    }
    return value;
  }, space);
}

export function stringifyWithRefs(obj: any, space?: string | number) {
  const objects = new Map(); // object -> path
  return JSON.stringify(obj, function (key, value) {
    // if(isObjectLike(value) && safeStringify(value).length>300) return '[redacted]'
    if (typeof value === 'object' && value !== null) {
      if (objects.has(value)) {
        return {$ref: objects.get(value)};
      }
      const path = this && this !== value ? (objects.get(this) || '$') + (key ? '.' + key : '') : '$';
      if (path.split('.').length > 3 || value.length>100) return '[redacted]'
      objects.set(value, path);
    }
    return value;
  }, space);
}
