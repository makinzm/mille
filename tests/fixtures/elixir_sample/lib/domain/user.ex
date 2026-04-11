defmodule MyApp.Domain.User do
  defstruct [:id, :name, :email]

  def new(id, name, email) do
    %__MODULE__{id: id, name: name, email: email}
  end

  def create(attrs) do
    %__MODULE__{
      id: attrs[:id],
      name: attrs[:name],
      email: attrs[:email]
    }
  end
end
