import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { ConnectionConfig, Driver, SslMode, errorMessage, ipc } from "@/lib/ipc";

const DEFAULT_PORTS: Record<Driver, number> = { postgres: 5432, mysql: 3306 };

export function emptyConfig(): ConnectionConfig {
  return {
    id: crypto.randomUUID(),
    name: "",
    driver: "postgres",
    host: "localhost",
    port: 5432,
    username: "",
    database: "",
    sslMode: "prefer",
  };
}

interface Props {
  initial: ConnectionConfig;
  isNew: boolean;
  onSaved: () => void;
  onDeleted: () => void;
}

export function ConnectionForm({ initial, isNew, onSaved, onDeleted }: Props) {
  const [config, setConfig] = useState(initial);
  const [password, setPassword] = useState("");
  const [status, setStatus] = useState<{ kind: "idle" | "ok" | "error" | "busy"; text: string }>({
    kind: "idle",
    text: "",
  });

  const patch = (p: Partial<ConnectionConfig>) => setConfig((c) => ({ ...c, ...p }));

  const passwordOrNull = password === "" ? null : password;

  const test = async () => {
    setStatus({ kind: "busy", text: "Testing..." });
    try {
      await ipc.connectionTest(config, passwordOrNull);
      setStatus({ kind: "ok", text: "Connection OK" });
    } catch (e) {
      setStatus({ kind: "error", text: errorMessage(e) });
    }
  };

  const save = async () => {
    setStatus({ kind: "busy", text: "Saving..." });
    try {
      await ipc.connectionSave(config, passwordOrNull);
      setStatus({ kind: "ok", text: "Saved" });
      onSaved();
    } catch (e) {
      setStatus({ kind: "error", text: errorMessage(e) });
    }
  };

  const remove = async () => {
    try {
      await ipc.connectionDelete(config.id);
      onDeleted();
    } catch (e) {
      setStatus({ kind: "error", text: errorMessage(e) });
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-3">
        <div className="col-span-2">
          <Label htmlFor="name">Name</Label>
          <Input id="name" value={config.name} onChange={(e) => patch({ name: e.target.value })} />
        </div>
        <div>
          <Label>Driver</Label>
          <Select
            value={config.driver}
            onValueChange={(v: Driver) => patch({ driver: v, port: DEFAULT_PORTS[v] })}
          >
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="postgres">PostgreSQL</SelectItem>
              <SelectItem value="mysql">MySQL / MariaDB</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div>
          <Label>SSL</Label>
          <Select value={config.sslMode} onValueChange={(v: SslMode) => patch({ sslMode: v })}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="disable">Disable</SelectItem>
              <SelectItem value="prefer">Prefer</SelectItem>
              <SelectItem value="require">Require</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div>
          <Label htmlFor="host">Host</Label>
          <Input id="host" value={config.host} onChange={(e) => patch({ host: e.target.value })} />
        </div>
        <div>
          <Label htmlFor="port">Port</Label>
          <Input
            id="port"
            type="number"
            value={config.port}
            onChange={(e) => patch({ port: Number(e.target.value) || 0 })}
          />
        </div>
        <div>
          <Label htmlFor="username">User</Label>
          <Input id="username" value={config.username} onChange={(e) => patch({ username: e.target.value })} />
        </div>
        <div>
          <Label htmlFor="password">Password</Label>
          <Input
            id="password"
            type="password"
            value={password}
            placeholder={isNew ? "" : "(unchanged, stored in Keychain)"}
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>
        <div className="col-span-2">
          <Label htmlFor="database">Database</Label>
          <Input id="database" value={config.database} onChange={(e) => patch({ database: e.target.value })} />
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Button variant="outline" onClick={test} disabled={status.kind === "busy"}>Test</Button>
        <Button onClick={save} disabled={status.kind === "busy"}>Save</Button>
        {!isNew && (
          <Button variant="destructive" onClick={remove}>Delete</Button>
        )}
        <span
          className={
            status.kind === "error" ? "text-sm text-red-500" : "text-sm text-muted-foreground"
          }
        >
          {status.text}
        </span>
      </div>
    </div>
  );
}
