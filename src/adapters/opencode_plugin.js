// Tells the atrium holding this opencode what it is doing, in the lines
// `atrium hook` sends. atrium writes this file and hands it over through
// OPENCODE_CONFIG_CONTENT, so an opencode started anywhere else never loads it.
import net from "node:net"

const socket = process.env.ATRIUM_SOCK
const agent = process.env.ATRIUM_AGENT_ID
// Taken back out, so an opencode that one of this one's tools starts neither
// loads this again nor reports as the agent that started it.
delete process.env.ATRIUM_SOCK
delete process.env.ATRIUM_AGENT_ID
delete process.env.OPENCODE_CONFIG_CONTENT

// One connection for the life of the process: lines on one arrive in the order
// they were written, which separate connections would not promise.
let connection

function send(word) {
  if (!socket || !agent) return
  try {
    if (!connection || connection.destroyed) {
      connection = net.createConnection(socket)
      // An atrium that has gone away is not worth taking opencode down with it.
      connection.on("error", () => {
        connection = undefined
      })
      // Nor worth keeping opencode running once it wants to exit.
      connection.unref()
    }
    connection.write(`${agent}\t${word}\t${Date.now()}\n`)
  } catch {
    connection = undefined
  }
}

export default {
  id: "atrium",
  server: async () => {
    // Which sessions a task started, by the session that started them. One
    // never seen is the one you are talking to.
    const parents = new Map()
    // Everything waiting on you, from any session: a question from a task
    // holds the whole agent up as surely as one from the session itself.
    const pending = new Set()
    let busy = false
    let failed = false
    let stopped = false
    const isTask = (id) => Boolean(parents.get(id))

    function handle({ type, properties: p = {} }) {
      switch (type) {
        case "session.created":
        case "session.updated":
          if (p.info) parents.set(p.info.id, p.info.parentID || null)
          return
        case "permission.asked":
        case "question.asked":
          if (pending.size === 0) send("PermissionRequest")
          pending.add(p.id)
          return
        case "permission.replied":
        case "question.replied":
        case "question.rejected":
          if (!pending.delete(p.requestID)) return
          if (type === "question.rejected" || p.reply === "reject") stopped = true
          if (pending.size === 0) send("PostToolUse")
          return
        case "session.error":
          if (!p.sessionID || isTask(p.sessionID)) return
          if (p.error && p.error.name === "MessageAbortedError") stopped = true
          else failed = true
          return
        case "session.status": {
          if (!p.sessionID || isTask(p.sessionID)) return
          const state = p.status && p.status.type
          if (state === "busy" || state === "retry") {
            // Busy is said again at every step. Once already busy it only
            // means the error or the refusal before it did not end the turn.
            if (busy) {
              failed = false
              stopped = false
              return
            }
            busy = true
            if (pending.size === 0) send("UserPromptSubmit")
            return
          }
          if (state === "idle" && busy) {
            busy = false
            // An interrupt drops whatever was waiting without a word.
            pending.clear()
            send(failed ? "StopFailure" : stopped ? "Interrupt" : "Stop")
            failed = false
            stopped = false
          }
          return
        }
      }
    }

    return {
      event: async ({ event }) => {
        try {
          handle(event)
        } catch {
          // A status update is never worth an error inside opencode.
        }
      },
    }
  },
}
