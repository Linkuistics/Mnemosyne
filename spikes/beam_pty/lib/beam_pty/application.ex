defmodule BeamPty.Application do
  @moduledoc false
  use Application

  @impl true
  def start(_type, _args) do
    {:ok, _} = :application.ensure_all_started(:erlexec)
    Supervisor.start_link([], strategy: :one_for_one, name: BeamPty.Supervisor)
  end
end
