const maxTimeout = 2147483647;

export type ICbRef = Buffer;

// LibVCX function which accepts a callback and rejects with a return code if needed
export type ICreateFFICallbackPromiseFn<T> = (
  resolve: (value: T | PromiseLike<T>) => void,
  reject: (reason?: number | string | Error) => void,
  cbRef: ICbRef,
) => void;

export type ICreateFFICallbackPromiseCb<T> = (
  resolve: (value: T | PromiseLike<T>) => void,
  reject: (reason?: number | string | Error) => void,
) => ICbRef;

export const createFFICallbackPromise = <T>(
  fn: ICreateFFICallbackPromiseFn<T>,
  cb: ICreateFFICallbackPromiseCb<T>,
): Promise<T> => {
  let cbRef = null;
  // TODO: Research why registering a callback doesn't keep parent thread alive https://github.com/node-ffi/node-ffi
  const processKeepAliveTimer = setTimeout(() => undefined, maxTimeout);
  return new Promise<T>((resolve, reject) => fn(resolve, reject, (cbRef = cb(resolve, reject))))
    .then((res) => {
      cbRef = null;
      clearTimeout(processKeepAliveTimer);
      return res;
    })
    .catch((err) => {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      cbRef = null;
      clearTimeout(processKeepAliveTimer);
      throw err;
    });
};
