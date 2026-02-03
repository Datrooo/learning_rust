export type Todo = {
  id: number;
  task: string;
  is_finished: boolean;
};

export type CreateTodo = {
  task: string;
  is_finished?: boolean;
};

export type UpdateTodo = {
  task?: string;
  is_finished?: boolean;
};

async function http<T>(input: RequestInfo, init?: RequestInit): Promise<T> {
  const res = await fetch(input, init);

  // DELETE может вернуть 204 без тела — это важно
  if (res.status === 204) {
    return undefined as T;
  }

  // если ошибка — попробуем вытащить текст/JSON
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`HTTP ${res.status}: ${text || res.statusText}`);
  }

  return res.json() as Promise<T>;
}

export function listTodos(): Promise<Todo[]> {
  return http<Todo[]>("/todos");
}

export function createTodo(data: CreateTodo): Promise<Todo> {
  return http<Todo>("/todos", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export function updateTodo(id: number, data: UpdateTodo): Promise<Todo> {
  return http<Todo>(`/todos/${id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export function deleteTodo(id: number): Promise<void> {
  return http<void>(`/todos/${id}`, { method: "DELETE" });
}