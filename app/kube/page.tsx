"use client";

import Button from '@/components/ui/button';
import {Combobox, ValueData} from '@/components/ui/combobox';
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from '@/components/ui/drawer';
import {PageTitle} from '@/components/ui/typography';
import {RefreshCwIcon} from 'lucide-react';
import {useEffect, useMemo} from 'react';
import {toast} from 'sonner';
import {Command} from 'tauri-plugin-shellx-api';
import {loadKubeConfig, NamedContext} from './api';
import {debounce} from 'lodash';
import {useKubeStore} from '@/stores/kube';
import {HoverCard, HoverCardContent} from '@/components/ui/hover-card';
import {HoverCardTrigger} from '@radix-ui/react-hover-card';

function DrawerDemo() {
  return (
    <Drawer>
      <DrawerTrigger>Open</DrawerTrigger>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>Are you absolutely sure?</DrawerTitle>
          <DrawerDescription>This action cannot be undone.</DrawerDescription>
        </DrawerHeader>
        <DrawerFooter>
          <Button>Submit</Button>
          <DrawerClose>
            <Button variant="outline">Cancel</Button>
          </DrawerClose>
        </DrawerFooter>
      </DrawerContent>
    </Drawer>
  );
}

function ContextDropdown({
  contexts,
  onChange,
}: {
  contexts: ValueData<NamedContext>[];
  onChange?: (context: NamedContext) => void;
}) {
  const { currentContext, setCurrentContext } = useKubeStore();
  return (
    <Combobox
      data={contexts}
      idFn={(c) => c.name}
      displayFn={(item) =>
        `${item.name} (${item.context.namespace || "default"})`
      }
      value={contexts.find((c) => c.value.name === currentContext)?.value}
      placeholder="Context"
      onChange={(v) => {
        const selected = contexts.find((c) => c.value.name === v);
        onChange && selected && onChange(selected.value);
        setCurrentContext?.(selected?.value.name!);
      }}
    />
  );
}

function ContextInfo() {
  const { currentContext, kube } = useKubeStore();
  const ctx = useMemo(() => {
    if (currentContext && kube) {
      return kube.contexts.find((c) => c.name === currentContext);
    }
  }, [currentContext, kube]);
  const user = useMemo(() => {
    if (currentContext && kube) {
      return kube.users.find((c) => c.name === ctx?.context.user);
    }
  }, [ctx, kube]);
  const cluster = useMemo(() => {
    if (currentContext && kube) {
      return kube.clusters.find((c) => c.name === ctx?.context.cluster);
    }
  }, [ctx, kube]);
  return (
    <div className="flex flex-col text-sm text-gray-400">
      <div>
        Context:{" "}
        <div className="text-gray-600 inline-block">
          <HoverInfo text={ctx?.name || ""}>
            <div className="font-mono text-xs">
              <span>Namespace: {ctx?.context.namespace || "default"}</span>
            </div>
          </HoverInfo>
        </div>{" "}
        <b className="text-blue-400 ml-2">
          ({ctx?.context.namespace || "default"})
        </b>
      </div>
      <div>
        Cluster:{" "}
        <span className="text-gray-600">
          <HoverInfo text={cluster?.name || ""}>
            <div className="font-mono text-xs">
              <span>Server: {cluster?.cluster.server}</span>
            </div>
          </HoverInfo>
        </span>
      </div>
      <div className="flex items-start gap-2">
        <span>User: </span>
        {(user?.user.exec && (
          <HoverInfo text={user?.name || ""}>
            <div className="font-mono text-xs">
              <span>
                Command: {user.user.exec.command}
                {" " + user.user.exec.args?.join(" ")}
              </span>
              {user.user.exec.env && (
                <div className="flex flex-col">
                  <span>Environment:</span>
                  <ul className="list-inside list-none ml-2">
                    {user.user.exec.env.map((e) => (
                      <li key={e.name}>
                        {e.name}: {e.value}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          </HoverInfo>
        )) || <span className="text-gray-600">{user?.name}</span>}
      </div>
    </div>
  );
}

function HoverInfo({
  text,
  children,
}: {
  text: string;
  children: React.ReactNode;
}) {
  return (
    <HoverCard>
      <HoverCardTrigger>
        <span className="text-blue-600">{text}</span>
      </HoverCardTrigger>
      <HoverCardContent className="ml-0 md:ml-16 w-[90vw] md:w-[100%]">
        {children}
      </HoverCardContent>
    </HoverCard>
  );
}

export default function KubePage() {
  const { kube, setKube } = useKubeStore();
  const debouncedRefresh = debounce(() => {
    loadKubeConfig().then((k) => {
      // console.info("KubeConfig:", x);
      setKube(k);
      console.debug(
        `Loaded ${k?.clusters.length} clusters, current context: ${k?.currentContext}`
      );
      return k;
    });
  }, 500);

  useEffect(() => {
    debouncedRefresh();
  }, []);

  return (
    <div className="flex gap-2 flex-col p-2">
      <PageTitle
        rightSection={
          <div className="flex gap-2 items-center">
            <Button
              className="px-1 py-1 w-8 text-gray-500 bg-gray-200 rounded-full hover:bg-gray-300"
              onClick={debouncedRefresh}
            >
              <RefreshCwIcon />
            </Button>
            <ContextDropdown
              contexts={kube?.contexts.map((c) => ({ value: c })) || []}
              onChange={(ctx) => {
                toast.success(`Switched to context: ${ctx.name}`);
              }}
            />
          </div>
        }
      >
        Kubernetes
      </PageTitle>
      <div className="flex flex-col gap-2">
        <ContextInfo />
        {/* <DrawerDemo /> */}
        <Button
          onClick={() => {
            const cmd = Command.create("aws", [
              "sts",
              "get-caller-identity",
              "--profile",
              "CloudEngineer-411632713503",
            ]);
            const res = cmd
              .execute()
              .then((x) => {
                if (x.code !== 0) {
                  throw x;
                }
                console.info("Response:", x.stdout || x.stderr);
                return `${x.code}: ${x.stdout || x.stderr}`;
              })
              .catch((x) => {
                console.error("Error:", x.stderr);
                throw x.stderr || x?.toString();
              });
            toast.promise(res, {
              loading: "Checking AWS",
              success: (d) => d || "OK",
              error: (e) => e || "Failed!",
            });
          }}
        >
          Check AWS
        </Button>
      </div>
    </div>
  );
}
