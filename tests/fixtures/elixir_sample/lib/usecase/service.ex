defmodule MyApp.Usecase.Service do
  alias MyApp.Domain.User

  def create_user(attrs) do
    user = User.create(attrs)
    {:ok, user}
  end
end
