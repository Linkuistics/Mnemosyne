defmodule BeamPty.MixProject do
  use Mix.Project

  def project do
    [
      app: :beam_pty,
      version: "0.1.0",
      elixir: "~> 1.19",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {BeamPty.Application, []}
    ]
  end

  defp deps do
    [
      {:erlexec, "~> 2.0"},
      {:jason, "~> 1.4"}
    ]
  end
end
