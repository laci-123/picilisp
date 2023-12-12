(export '(repl read-eval-print))

(defun -pretty-print-syntax-error (error)
  ""
  (concat "message: \n"
          (. error 'message)
          "\n at: "
          (let (location (. error 'location))
            (concat (print (. location 'file))
                    ":"
                    (print (. location 'line))
                    ":"
                    (print (. location 'column))))
          "\n"))

(defun pretty-print-error (error)
  "Print `erorr` in a more human-readable format.
If `error` is not a valid property-list then just simply print it using the `print` function."
  (if error
      (try
       (let (key (car error), value (car (cdr error)))
         (concat (print key)
                 ":\n"
                 (if (or (= key 'symbol) (= key 'argument-value))
                     (let (md (get-metadata value))
                       (concat (print value)
                               "\n at: "
                               (print (. md 'file))
                               (if (and (. md 'line) (. md 'column))
                                   (concat ":" (print (. md 'line)) ":" (print (. md 'column)))
                                   "")))
                     (print value))
                 "\n\n"
                 (if (= value 'syntax-error)
                     (-pretty-print-syntax-error (. error 'details))
                     (pretty-print-error         (cdr (cdr error))))))
       (catch-all (lambda (_) (print error))))
      ""))

(defun repl (prompt initial-input)
  "(R)ead an expression from standard input,
(E)valuated it,
(P)rint the result to standard output,
then repeat (or (L)oop) from the beginning.
Stop the loop when end of input (EOF) is reached."
  (try
   (let (current-input (concat initial-input (input prompt)))
     (let (read-result (read current-input))
       (let (read-status (. read-result 'status))
         (case ((= read-status 'invalid)    (throw 'kind 'invalid-string, 'source 'repl))
               ((= read-status 'nothing)    (repl prompt nil))
               ((= read-status 'incomplete) (repl "... " current-input))
               ((= read-status 'error)      (throw 'kind 'syntax-error, 'source 'repl, 'details (. read-result 'error)))
               ((= read-status 'ok)         (block (output (print (eval (. read-result 'result))))
                                                          (repl ">>> " nil)))
               (t                                  (throw 'kind 'unknown-read-status, 'source (qoute repl), 'read-status read-status))))))
   (catch eof
     (lambda (_) (block (output "")
                        'ok)))
   (catch-all
    (lambda (error) (block (output (concat "UNHANDLED ERROR:\n\n" (pretty-print-error error)))
                           (repl ">>> " nil))))))

(defun read-eval-print (string pretty-print-errors?)
  "Read a string, evaluate it then print it into a string.
If a signal is emmited during read evaluate or print then pretty-print it then forward it."
  (try
   (let (read-result (read string))
     (let (read-status (. read-result 'status))
       (case ((= read-status 'invalid)    (throw 'kind 'invalid-string, 'source 'repl))
             ((= read-status 'nothing)    "")
             ((= read-status 'incomplete) (throw 'kind 'syntax-error,   'source 'read-eval-print, 'details 'incomplete-input))
             ((= read-status 'error)      (throw 'kind 'syntax-error,   'source 'repl,            'details (. read-result 'error)))
             ((= read-status 'ok)         (print (eval (. read-result 'result))))
             (t                           (throw 'kind 'unknown-read-status, 'source (qoute repl), 'read-status read-status)))))
  (catch-all
   (lambda (error) (if pretty-print-errors?
                       (signal (pretty-print-error error))
                       (signal error))))))
