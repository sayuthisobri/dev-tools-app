import {run} from '@/lib/utils'

export enum HTTPMethod {
  GET = 'GET',
  POST = 'POST',
  PUT = 'PUT',
  PATCH = 'PATCH',
  DELETE = 'DELETE',
  OPTIONS = 'OPTIONS',
  HEAD = 'HEAD',
}

export enum ContentType {
  JSON = 'application/json',
  Form = 'application/x-www-form-urlencoded',
  Multipart = 'multipart/form-data',
  XML = 'application/xml',
  Plain = 'text/plain',
}

export interface KVParam {
  [key: string]: unknown;

  key: string;
  value: string;
  enabled: boolean;
}

export interface HTTPRequest {
  [key: string]: unknown;

  method?: string;
  url: string;
  body?: string;
  contentType?: string;
  headers?: KVParam[];
  query?: KVParam[];
  auth?: KVParam[];
}

export interface HTTPResponse {
  [key: string]: unknown;

  // response id
  id?: string;
  // api id
  api: string;
  req: HTTPRequest;
  length: number;
  latency: number;
  status: number;
  headers: Map<string, string[]>;
  body: string;
  // stats: HTTPStats;
}

export interface RequestTimeout {
  [key: string]: unknown;

  connect: number;
  write: number;
  read: number;
}

function is_json(str: string) {
  const value = str.trim();
  if (value.length < 2) {
    return false;
  }
  const key = value[0] + value[value.length - 1];
  return key === '{}' || key === '[]';
}

export async function request(req: HTTPRequest): Promise<HTTPResponse> {
  if (!req.headers) {
    req.headers = [];
  }
  if (!req.query) {
    req.query = [];
  }
  if (!req.auth) {
    req.auth = [];
  }
  const method = req.method || HTTPMethod.GET;
  let body = req.body || '';
  let contentType = req.contentType || '';
  // if (
  //   ![HTTPMethod.POST, HTTPMethod.PATCH, HTTPMethod.PUT].includes(
  //     method as HTTPMethod,
  //   )
  // ) {
  //   body = "";
  //   contentType = "";
  // }
  // body = await convertBody(collection, body);
  // if (body && contentType === ContentType.Form) {
  //   const arr = JSON.parse(body) as KVParam[];
  //   const result: string[] = [];
  //   arr.forEach((item) => {
  //     if (!item.enabled) {
  //       return;
  //     }
  //     result.push(
  //       `${window.encodeURIComponent(item.key)}=${window.encodeURIComponent(
  //         item.value,
  //       )}`,
  //     );
  //   });
  //   body = result.join("&");
  // }
  // if (body && contentType === ContentType.Multipart) {
  //   const data = await convertMultipartForm(body);
  //   contentType = data.headers["Content-Type"];
  //   body = data.body;
  // }
  const params = {
    method: method,
    url: req.url,
    body,
    contentType,
    headers: req.headers,
    query: req.query,
  };
  // await convertKVParams(collection, params.query);
  // await convertKVParams(collection, params.headers);


  const requestTimeout = {
    connect: 10,
    write: 120,
    read: 300,
  };
  // eslint-disable-next-line
  // @ts-ignore
  let resp: HTTPResponse = {};

  const startedAt = Date.now();
  type Params = Partial<{ req: HTTPRequest, timeout: RequestTimeout }>;

  resp = await run<HTTPResponse, Params>('http_request', {
    req: params,
    // timeout: requestTimeout,
  });
  if (resp.latency <= 0) {
    resp.latency = 1;
  }
  return resp;
}

