defmodule MyApp.Infrastructure.Repo do
  alias MyApp.Domain.User
  alias Ecto.Repo

  def find_user(id) do
    {:ok, %User{id: id, name: "test", email: "test@example.com"}}
  end
end
