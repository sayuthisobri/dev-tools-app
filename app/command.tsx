import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command"
import React from "react"
import { useRouter } from 'next/navigation'; 

export function CommandMenu() {
  const [open, setOpen] = React.useState(false);
  const router = useRouter();
  React.useEffect(() => {
    const onKeydown = (e: KeyboardEvent) => {
      if (e.key === "a" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen((open) => !open);
      }
    };
    document.addEventListener("keydown", onKeydown);
    return () => document.removeEventListener("keydown", onKeydown);
  }, []);

  const links : { name: string; href: string; keywords?: string[] }[] = [
    { name: "Dashboard", href: "/", keywords: ["home", "main"] },
    { name: "Kubernetes", href: "/kube", keywords: ["k8s", "kube"] },
    { name: "Network", href: "/network" },
    { name: "Databases", href: "/databases" },
    { name: "Http Client", href: "/http" },
    { name: "SOAP", href: "/soap" },
    { name: "Files", href: "/files" },
    { name: "Settings", href: "/settings" },
  ];

  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <CommandInput placeholder="Type a command or search..." />
      <CommandList>
        <CommandEmpty>No results found 😅</CommandEmpty>
        <CommandGroup heading="Suggestions">
          {links.map((link) => (
            <CommandItem key={link.href} keywords={link.keywords || []}
              onSelect={() => {
                console.trace("push link", link.href);
                router.push(link.href);
                setOpen(false);
              }}
            >
              {link.name}
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  );
}
