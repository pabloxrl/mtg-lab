# Execute pinned controller decisions; never start its supervisor, tracker client,
# worker or app-server. The only tracker boundary is an injected in-memory fetch.
Application.ensure_all_started(:elixir)
Application.ensure_all_started(:logger)
Logger.configure(level: :error)
alias SymphonyElixir.{AgentRunner, Config, Orchestrator, Workflow, WorkflowStore}
alias SymphonyElixir.Orchestrator.State
alias SymphonyElixir.Tracker.Issue

Workflow.set_workflow_file_path(hd(System.argv()))
{:ok, _} = WorkflowStore.start_link()
settings = Config.settings!()
3 = settings.agent.max_concurrent_agents
12 = settings.agent.max_turns
300_000 = settings.agent.max_retry_backoff_ms
600_000 = settings.codex.turn_timeout_ms
600_000 = settings.codex.stall_timeout_ms

i = %Issue{id: "189", identifier: "GH-189", title: "isolated smoke", state: "open",
           labels: ["agent-ready"], dispatchable: true}
s = %State{max_concurrent_agents: 1,
           codex_totals: %{input_tokens: 0, output_tokens: 0, total_tokens: 0, seconds_running: 0}}
true = Orchestrator.should_dispatch_issue_for_test(i, s)
{:continue, ^i} = AgentRunner.continue_with_issue_for_test(i, fn ["189"] -> {:ok, [i]} end)
# Pause/removal and cancellation are controller routing controls. Held/blocked
# and program pause controls are additional worker workflow gates, not native
# controller labels; the operator runbook requires removing ready when holding.
for stopped <- [%{i | labels: []}, %{i | state: "closed"}, %{i | dispatchable: false},
                %{i | labels: ["agent-held"]}, %{i | labels: ["agent-blocked"]}] do
  false = Orchestrator.should_dispatch_issue_for_test(stopped, s)
  {:done, ^stopped} = AgentRunner.continue_with_issue_for_test(i, fn _ -> {:ok, [stopped]} end)
end
{:error, {:issue_state_refresh_failed, :unavailable}} =
  AgentRunner.continue_with_issue_for_test(i, fn _ -> {:error, :unavailable} end)

ref = make_ref()
entry = %{ref: ref, pid: self(), identifier: i.identifier, issue: i,
          retry_attempt: 80, started_at: DateTime.utc_now(), session_id: "smoke",
          codex_total_tokens: 0, codex_input_tokens: 0, codex_output_tokens: 0}
busy = %{s | running: %{i.id => entry}, claimed: MapSet.new([i.id])}
false = Orchestrator.should_dispatch_issue_for_test(%{i | id: "190"}, busy)

# Parallel admission uses the packaged controller, including duplicate claims.
parallel = %{busy | max_concurrent_agents: settings.agent.max_concurrent_agents}
false = Orchestrator.should_dispatch_issue_for_test(i, parallel)
true = Orchestrator.should_dispatch_issue_for_test(%{i | id: "190", identifier: "GH-190"}, parallel)
second = %{entry | issue: %{i | id: "190", identifier: "GH-190"}, identifier: "GH-190"}
third = %{entry | issue: %{i | id: "191", identifier: "GH-191"}, identifier: "GH-191"}
parallel = %{parallel | running: %{"189" => entry, "190" => second},
                         claimed: MapSet.new(["189", "190"])}
true = Orchestrator.should_dispatch_issue_for_test(%{i | id: "191", identifier: "GH-191"}, parallel)
parallel = %{parallel | running: Map.put(parallel.running, "191", third),
                         claimed: MapSet.new(["189", "190", "191"])}
false = Orchestrator.should_dispatch_issue_for_test(%{i | id: "192", identifier: "GH-192"}, parallel)

# Actual worker DOWN handling: normal completion (including max_turns return)
# schedules a fresh continuation; failures retain bounded exponential backoff
# even at large lifetime counts. No wall-clock sleeps or live services.
for {reason, previous, expected_attempt, delay} <- [
      {:normal, 80, 1, 1_000}, {:shutdown, nil, 1, 10_000},
      {:shutdown, 1, 2, 20_000}, {:shutdown, 80, 81, 300_000}] do
  busy = %{busy | running: %{i.id => %{entry | retry_attempt: previous}}}
  before = System.monotonic_time(:millisecond)
  {:noreply, after_state} = Orchestrator.handle_info({:DOWN, ref, :process, self(), reason}, busy)
  after_time = System.monotonic_time(:millisecond)
  %{} = after_state.running
  true = map_size(after_state.running) == 0
  true = map_size(after_state.blocked) == 0
  retry = Map.fetch!(after_state.retry_attempts, i.id)
  ^expected_attempt = retry.attempt
  true = retry.due_at_ms >= before + delay
  true = retry.due_at_ms <= after_time + delay
  Process.cancel_timer(retry.timer_ref)
end
# Exercise the real runner's 12-turn return, twice, with only filesystem/hook
# and Codex process boundaries replaced. Hook execution has its own real-code
# tests; the scheduler/runner and workflow parser remain the packaged modules.
Code.compiler_options(ignore_module_conflict: true)
defmodule SymphonyElixir.Workspace do
  def create_for_issue(_, _), do: {:ok, System.tmp_dir!()}
  def run_before_run_hook(_, _, _), do: :ok
  def run_after_run_hook(_, _, _), do: :ok
end
defmodule SymphonyElixir.Codex.AppServer do
  def start_session(_, _), do: {:ok, %{}}
  def run_turn(_, _, _, _) do
    Process.put(:smoke_turns, Process.get(:smoke_turns, 0) + 1)
    {:ok, %{session_id: "isolated"}}
  end
  def stop_session(_), do: Process.put(:smoke_stops, Process.get(:smoke_stops, 0) + 1)
end
for expected <- [12, 24] do
  :ok = AgentRunner.run(i, nil, issue_state_fetcher: fn _ -> {:ok, [i]} end)
  ^expected = Process.get(:smoke_turns)
end
2 = Process.get(:smoke_stops)
IO.puts("credential-free-controller-continuation-ok")
