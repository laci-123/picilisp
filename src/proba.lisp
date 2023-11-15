(export '(abc show-secret do-something-bad))

(define 'abc 123 "This is abc")

(define 'secret 789 "This is secret")

(defun -show-secret (n)
  ""
  (add secret n))

(defun show-secret ()
  "Reveale the secret!"
  ((lambda (n) (-show-secret n)) 1))

(defun do-something-bad ()
  "Do something bad"
  (try
   (list 1 2 (add 3 x))
   (catch unbound-symbol
     (lambda(error) (concat (print "This is the bad one: ")
                            (print (. error 'symbol)))))))
