// src/pages/NotFoundPage.tsx

export function NotFoundPage({ title = "Page not found", message = "There's nothing at this address." }: { title?: string; message?: string }) {
  return (
    <div class="container page narrow">
      <div class="panel empty-state">
        <h1 class="page-title">{title}</h1>
        <p class="muted">{message}</p>
        <a class="btn btn-primary" href="/">
          Back to variants
        </a>
      </div>
    </div>
  );
}
