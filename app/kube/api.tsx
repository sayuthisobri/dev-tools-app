import { run } from '@/lib/utils'
import { invoke } from "@tauri-apps/api/core"

export const loadKubeConfig: (
  path?: string
) => Promise<KubeConfig | undefined> = async (path = "~/.kube/config") => {
  return await run<KubeConfig>("load_kubeconfig", {
    path,
  }).then((res: any) => {
    if (res != null && typeof res === "object") {
      res.currentContext = res["current-context"];
      delete res["current-context"];

      res.users = res.users?.filter((u:any)=>!!u.user).map((u:any)=>{
        u.user.clientCertificate = u.user['client-certificate'];
        delete u.user['client-certificate'];
        u.user.clientKey = u.user['client-key'];
        delete u.user['client-key'];
        return u;
      })
      return res as KubeConfig;
    }
    throw new Error("Invalid KubeConfig format");
  });
};

export interface KubeConfig {
  apiVersion: string;
  clusters: NamedCluster[];
  contexts: NamedContext[];
  currentContext: string;
  kind: string;
  preferences: Record<string, unknown>;
  users: NamedUser[];
}

export interface NamedCluster {
  name: string;
  cluster: ClusterInfo;
}

export interface NamedContext {
  name: string;
  context: ContextInfo;
}

export interface ContextInfo {
  cluster: string;
  user: string;
  namespace?: string;
}

export interface UserExecConfig {
  apiVersion: string;
  command: string;
  args?: string[];
  env?: { name: string; value: string }[];
}

export interface NamedUser {
  name: string;
  user: User;
}

export interface User {
  token?: string;
  tokenFile?: string;
  clientCertificate?: string;
  clientKey?: string;
  clientCertificateData?: string; // base64-encoded
  clientKeyData?: string; // base64-encoded
  username?: string;
  password?: string;
  exec?: UserExecConfig;
  // auth-provider, exec, etc. can be added later
}

export interface ClusterInfo {
  server: string;
  certificateAuthority?: string;
  certificateAuthorityData?: string; // base64-encoded
  insecureSkipTlsVerify?: boolean;
}
